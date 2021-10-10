use std::{pin::Pin, task::{Context, Poll}};
use tokio::{spawn, time::{self, Duration}, sync::mpsc::{self, error::TryRecvError}};
use futures::{Stream, StreamExt, SinkExt, ready};
use rand::{seq::SliceRandom, thread_rng as rng};
use tokio_tungstenite::{connect_async, tungstenite::{self, protocol::Message}};
use crate::{
    config::HEARTBEAT_RATE_SEC,
    api::room::HostsInfo,
    feed::package::Package
};

pub struct FeedStream {
    ws: futures::stream::SplitStream<tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>>,
    error: mpsc::Receiver<tungstenite::Error>,
}

impl FeedStream {
    pub async fn connect(roomid: u32) -> Self {
        let (error_sender, error_receiver) = mpsc::channel::<tungstenite::Error>(2);

        let hosts_info = HostsInfo::call(roomid).await.unwrap(); // !
        let host = &hosts_info.host_list.choose(&mut rng()).unwrap();
        let url = format!("wss://{}:{}/sub", host.host, host.wss_port);

        let (ws, _) = connect_async(url.as_str()).await.unwrap(); // !
        let (mut sender, receiver) = ws.split();

        let init = Message::Binary(Package::create_init_request(roomid, hosts_info.token).encode());
        sender.send(init).await.unwrap(); // !

        spawn(async move {
            let heartbeat = Message::Binary(Package::HeartbeatRequest().encode());
            let mut interval = time::interval(Duration::from_secs(HEARTBEAT_RATE_SEC));
            loop {
                interval.tick().await;
                if let Err(error) = sender.send(heartbeat.clone()).await {
                    if let Err(_) = error_sender.send(error).await {
                        break
                    }
                }
            }
        });

        Self {
            ws: receiver,
            error: error_receiver,
        }
    }
}

impl Stream for FeedStream {
    type Item = Vec<u8>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        // It works but I don't know why
        let message = ready!(Pin::new(&mut self.ws).poll_next(cx));
        let heartbeat_error = self.error.try_recv();

        Poll::Ready(match heartbeat_error {
            Ok(error) => {
                eprintln!("FEEDSTREAM HEARTBEAT! {}", error);
                None
            },
            Err(TryRecvError::Empty) => match message {
                Some(Ok(Message::Binary(payload))) => Some(payload),
                Some(Ok(Message::Ping(payload))) => {
                    assert!(payload.is_empty());
                    eprintln!("FEEDSTREAM RECEIVED ENPTY PING");
                    None
                },
                Some(Ok(_)) => unreachable!(),
                Some(Err(error)) => {
                    eprintln!("FEEDSTREAM! {}", error);
                    None
                },
                None => None,
            },
            Err(TryRecvError::Disconnected) => unreachable!(),
        })
    }
}
