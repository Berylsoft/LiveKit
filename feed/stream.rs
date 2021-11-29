use std::{pin::Pin, task::{Context, Poll}};
use tokio::{
    spawn,
    time::{self, Duration},
    sync::mpsc::{self, error::TryRecvError},
    io::{Error as IoError, AsyncRead, AsyncWriteExt, ReadBuf},
    net::{TcpStream, tcp::OwnedReadHalf as TcpSocket},
};
use futures::{Stream, StreamExt, SinkExt, ready};
use rand::{seq::SliceRandom, thread_rng as rng};
use tokio_tungstenite::{
    connect_async,
    tungstenite::{protocol::Message, Error as WsError}
};
use livekit_api::feed::HostsInfo;
use crate::{
    config::{FEED_HEARTBEAT_RATE_SEC, FEED_TCP_BUFFER_SIZE},
    package::Package,
};

type WebSocket = futures::stream::SplitStream<tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>>;

pub struct FeedStream<Socket, Error> {
    roomid: u32,
    socket: Socket,
    error: mpsc::Receiver<Error>,
}

impl FeedStream<WebSocket, WsError> {
    pub async fn connect_ws(roomid: u32, hosts_info: HostsInfo) -> Result<Self, WsError> {
        let (error_sender, error_receiver) = mpsc::channel::<WsError>(2);

        let host = &hosts_info.host_list.choose(&mut rng()).unwrap();
        let (ws, _) = connect_async(format!("wss://{}:{}/sub", host.host, host.wss_port)).await?;
        let (mut sender, receiver) = ws.split();
        log::debug!("[{: >10}] (ws) connected", roomid);

        let init = Message::Binary(Package::create_init_request(roomid, "web".to_owned(), hosts_info.token).encode().unwrap());
        sender.send(init).await?;
        log::debug!("[{: >10}] (ws) sent: init", roomid);

        spawn(async move {
            let heartbeat = Message::Binary(Package::HeartbeatRequest().encode().unwrap());
            let mut interval = time::interval(Duration::from_secs(FEED_HEARTBEAT_RATE_SEC));
            loop {
                interval.tick().await;
                if let Err(error) = sender.send(heartbeat.clone()).await {
                    if let Err(_) = error_sender.send(error).await {
                        break
                    }
                }
                log::debug!("[{: >10}] (ws) sent: (heartbeat-thread) heartbeat", roomid);
            }
        });

        Ok(Self {
            roomid,
            socket: receiver,
            error: error_receiver,
        })
    }
}

impl Stream for FeedStream<WebSocket, WsError> {
    type Item = Vec<u8>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let message = ready!(Pin::new(&mut self.socket).poll_next(cx));
        let heartbeat_error = self.error.try_recv();

        match heartbeat_error {
            Ok(error) => {
                log::warn!("[{: >10}] (ws) close: (heartbeat-thread) caused by {:?}", self.roomid, error);
                Poll::Ready(None)
            },
            Err(TryRecvError::Empty) => {
                match message {
                    Some(Ok(Message::Binary(payload))) => {
                        log::debug!("[{: >10}] (ws) recv: message {}", self.roomid, payload.len());
                        Poll::Ready(Some(payload))
                    },
                    Some(Ok(Message::Ping(payload))) => {
                        if payload.is_empty() {
                            log::debug!("[{: >10}] (ws) recv: empty ping", self.roomid);
                        } else {
                            log::error!("[{: >10}] (ws) recv: non-empty ping {:?}", self.roomid, payload);
                        }
                        Poll::Pending
                    },
                    Some(Ok(message)) => {
                        log::error!("[{: >10}] (ws) recv: unexpected message type {:?}", self.roomid, message);
                        Poll::Pending
                    },
                    Some(Err(error)) => {
                        log::warn!("[{: >10}] (ws) close: caused by {:?}", self.roomid, error);
                        Poll::Ready(None)
                    },
                    None => {
                        log::warn!("[{: >10}] (ws) close: normally", self.roomid);
                        Poll::Ready(None)
                    },
                }
            },
            Err(TryRecvError::Disconnected) => {
                unreachable!()
            },
        }
    }
}

impl FeedStream<TcpSocket, IoError> {
    pub async fn connect_tcp(roomid: u32, hosts_info: HostsInfo) -> Result<Self, IoError> {
        let (error_sender, error_receiver) = mpsc::channel::<IoError>(2);

        let host = &hosts_info.host_list.choose(&mut rng()).unwrap();
        let tcp = TcpStream::connect((host.host.as_str(), host.port)).await?;
        let (receiver, mut sender) = tcp.into_split();
        log::debug!("[{: >10}] (tcp) connected", roomid);

        let init = Package::create_init_request(roomid, "web".to_owned(), hosts_info.token).encode().unwrap();
        sender.write_all(init.as_slice()).await?;
        log::debug!("[{: >10}] (tcp) sent: init", roomid);

        spawn(async move {
            let heartbeat = Package::HeartbeatRequest().encode().unwrap();
            let mut interval = time::interval(Duration::from_secs(FEED_HEARTBEAT_RATE_SEC));
            loop {
                interval.tick().await;
                if let Err(error) = sender.write_all(heartbeat.as_slice()).await {
                    if let Err(_) = error_sender.send(error).await {
                        break
                    }
                }
                log::debug!("[{: >10}] (tcp) sent: (heartbeat-thread) heartbeat", roomid);
            }
        });

        Ok(Self {
            roomid,
            socket: receiver,
            error: error_receiver,
        })
    }
}

impl Stream for FeedStream<TcpSocket, IoError> {
    type Item = Vec<u8>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let mut bytes = [0u8; FEED_TCP_BUFFER_SIZE];
        let mut readbuf = ReadBuf::new(&mut bytes);

        let message = ready!(Pin::new(&mut self.socket).poll_read(cx, &mut readbuf));
        let heartbeat_error = self.error.try_recv();

        match heartbeat_error {
            Ok(error) => {
                log::warn!("[{: >10}] (tcp) close: (heartbeat-thread) caused by {:?}", self.roomid, error);
                Poll::Ready(None)
            },
            Err(TryRecvError::Empty) => {
                match message {
                    Ok(()) => {
                        let payload = readbuf.filled().to_vec();
                        log::debug!("[{: >10}] (tcp) recv: message {}", self.roomid, payload.len());
                        Poll::Ready(Some(payload))
                    },
                    Err(error) => {
                        log::warn!("[{: >10}] (tcp) close: caused by {:?}", self.roomid, error);
                        Poll::Ready(None)
                    },
                }
            },
            Err(TryRecvError::Disconnected) => {
                unreachable!()
            },
        }
    }
}