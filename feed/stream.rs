use std::{pin::Pin, task::{Context, Poll}};
use futures::{Stream, StreamExt, SinkExt, ready};
use rand::{seq::SliceRandom, thread_rng as rng};
use tokio::{spawn, time::{self, Duration}, net::TcpStream};
use tokio::{
    io::{Error as IoError, AsyncRead, AsyncWriteExt, ReadBuf},
    net::{tcp::OwnedReadHalf as TcpStreamReceiver},
};
use tokio_tungstenite::{connect_async as connect_ws_stream, tungstenite::protocol::Message};
use livekit_api::feed::HostsInfo;
use crate::{config::*, payload::Payload, package::Package};

pub type WsStreamReceiver = futures::stream::SplitStream<tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<TcpStream>>>;
pub use tokio_tungstenite::tungstenite::Error as WsError;

pub struct FeedStream<T> {
    roomid: u32,
    rx: T,
}

pub type WsFeedStream = FeedStream<WsStreamReceiver>;

impl WsFeedStream {
    pub async fn connect_ws(roomid: u32, hosts_info: HostsInfo) -> Result<Self, WsError> {
        let host = hosts_info.host_list.choose(&mut rng()).unwrap();
        let (stream, _) = connect_ws_stream(format!("wss://{}:{}/sub", host.host, host.wss_port)).await?;
        let (mut tx, rx) = stream.split();
        log::debug!("[{: >10}] (ws) connected", roomid);

        let init = Message::Binary(Package::create_init_request(roomid, "web".to_owned(), hosts_info.token).encode().unwrap());
        tx.send(init).await?;
        log::debug!("[{: >10}] (ws) sent: init", roomid);

        spawn(async move {
            let heartbeat = Message::Binary(Package::HeartbeatRequest().encode().unwrap());
            let mut interval = time::interval(Duration::from_secs(FEED_HEARTBEAT_RATE_SEC));
            loop {
                interval.tick().await;
                if let Err(error) = tx.send(heartbeat.clone()).await {
                    log::warn!("[{: >10}] (ws) stop sending: (heartbeat-thread) caused by {:?}", roomid, error);
                    break;
                }
                log::debug!("[{: >10}] (ws) sent: (heartbeat-thread) heartbeat", roomid);
            }
        });

        Ok(Self {
            roomid,
            rx,
        })
    }
}

impl Stream for WsFeedStream {
    type Item = Option<Payload>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        Poll::Ready(match ready!(Pin::new(&mut self.rx).poll_next(cx)) {
            Some(Ok(message)) => Some(match message {
                Message::Binary(payload) => {
                    log::debug!("[{: >10}] (ws) recv: message {}", self.roomid, payload.len());
                    Some(Payload::new(payload))
                },
                Message::Ping(payload) => {
                    if payload.is_empty() {
                        log::debug!("[{: >10}] (ws) recv: empty ping", self.roomid);
                    } else {
                        log::error!("[{: >10}] (ws) recv: non-empty ping {:?}", self.roomid, payload);
                    }
                    None
                },
                message => {
                    log::error!("[{: >10}] (ws) recv: unexpected message type {:?}", self.roomid, message);
                    None
                },
            }),
            Some(Err(error)) => {
                log::warn!("[{: >10}] (ws) close: caused by {:?}", self.roomid, error);
                None
            },
            None => {
                log::warn!("[{: >10}] (ws) close: normally", self.roomid);
                None
            },
        })
    }
}

pub type TcpFeedStream = FeedStream<TcpStreamReceiver>;

impl TcpFeedStream {
    pub async fn connect_tcp(roomid: u32, hosts_info: HostsInfo) -> Result<Self, IoError> {
        let host = hosts_info.host_list.choose(&mut rng()).unwrap();
        let stream = TcpStream::connect((host.host.as_str(), host.port)).await?;
        let (rx, mut tx) = stream.into_split();
        log::debug!("[{: >10}] (tcp) connected", roomid);

        let init = Package::create_init_request(roomid, "web".to_owned(), hosts_info.token).encode().unwrap();
        tx.write_all(init.as_slice()).await?;
        log::debug!("[{: >10}] (tcp) sent: init", roomid);

        spawn(async move {
            let heartbeat = Package::HeartbeatRequest().encode().unwrap();
            let mut interval = time::interval(Duration::from_secs(FEED_HEARTBEAT_RATE_SEC));
            loop {
                interval.tick().await;
                if let Err(error) = tx.write_all(heartbeat.as_slice()).await {
                    log::warn!("[{: >10}] (tcp) stop sending: (heartbeat-thread) caused by {:?}", roomid, error);
                    break;
                }
                log::debug!("[{: >10}] (tcp) sent: (heartbeat-thread) heartbeat", roomid);
            }
        });

        Ok(Self {
            roomid,
            rx,
        })
    }
}

impl Stream for TcpFeedStream {
    type Item = Option<Payload>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let mut bytes = [0u8; FEED_TCP_BUFFER_SIZE];
        let mut readbuf = ReadBuf::new(&mut bytes);

        Poll::Ready(match ready!(Pin::new(&mut self.rx).poll_read(cx, &mut readbuf)) {
            Ok(()) => Some({
                let payload = readbuf.filled().to_vec();
                log::debug!("[{: >10}] (tcp) recv: message {}", self.roomid, payload.len());
                Some(Payload::new(payload))
            }),
            Err(error) => {
                log::warn!("[{: >10}] (tcp) close: caused by {:?}", self.roomid, error);
                None
            },
        })
    }
}
