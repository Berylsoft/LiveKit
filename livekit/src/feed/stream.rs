use std::{pin::Pin, task::{Context, Poll}};
use tokio::{spawn, time::{self, Duration}, sync::mpsc::{self, error::TryRecvError}};
use futures::{Stream, StreamExt, SinkExt, ready};
use rand::{seq::SliceRandom, thread_rng as rng};
use tokio_tungstenite::{connect_async, tungstenite::{protocol::Message, Error as WsError}};
use crate::{
    config::FEED_HEARTBEAT_RATE_SEC,
    api::room::HostsInfo,
    feed::package::Package,
};

pub struct FeedStream {
    roomid: u32,
    ws: futures::stream::SplitStream<tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>>,
    error: mpsc::Receiver<WsError>,
}

impl FeedStream {
    pub async fn connect(roomid: u32, hosts_info: HostsInfo) -> Result<Self, WsError> {
        let (error_sender, error_receiver) = mpsc::channel::<WsError>(2);

        let host = &hosts_info.host_list.choose(&mut rng()).unwrap();
        let (ws, _) = connect_async(format!("wss://{}:{}/sub", host.host, host.wss_port)).await?;
        let (mut sender, receiver) = ws.split();

        let init = Message::Binary(Package::create_init_request(roomid, hosts_info.token).encode());
        sender.send(init).await?;

        spawn(async move {
            let heartbeat = Message::Binary(Package::HeartbeatRequest().encode());
            let mut interval = time::interval(Duration::from_secs(FEED_HEARTBEAT_RATE_SEC));
            loop {
                interval.tick().await;
                if let Err(error) = sender.send(heartbeat.clone()).await {
                    if let Err(_) = error_sender.send(error).await {
                        break
                    }
                }
            }
        });

        Ok(Self {
            roomid,
            ws: receiver,
            error: error_receiver,
        })
    }
}

impl Stream for FeedStream {
    type Item = Vec<u8>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let message = ready!(Pin::new(&mut self.ws).poll_next(cx));
        let heartbeat_error = self.error.try_recv();

        match heartbeat_error {
            Ok(error) => {
                log::warn!("[{:010} FEEDSTREAM HEARTBEAT]! {}", self.roomid, error);
                Poll::Ready(None)
            },
            Err(TryRecvError::Empty) => {
                match message {
                    Some(Ok(Message::Binary(payload))) => {
                        Poll::Ready(Some(payload))
                    },
                    Some(Ok(Message::Ping(payload))) => {
                        if !payload.is_empty() {
                            log::error!("[{:010} FEEDSTREAM]!! recv non-empty ping {:?}", self.roomid, payload);
                        }
                        Poll::Pending
                    },
                    Some(Ok(message)) => {
                        log::error!("[{:010} FEEDSTREAM]!! recv unexpected message type {:?}", self.roomid, message);
                        Poll::Pending
                    },
                    Some(Err(error)) => {
                        log::warn!("[{:010} FEEDSTREAM]! {}", self.roomid, error);
                        Poll::Ready(None)
                    },
                    None => {
                        log::warn!("[{:010} FEEDSTREAM]! close normally", self.roomid);
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
