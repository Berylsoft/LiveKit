use std::{pin::Pin, task::{Context, Poll}};
use tokio::{spawn, time::{self, Duration}};
use futures::{Stream, StreamExt, SinkExt, stream::SplitStream, ready};
use rand::{seq::SliceRandom, thread_rng as rng};
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};
use crate::{
    config::HEARTBEAT_RATE_SEC,
    api::room::HostsInfo,
    feed::package::Package
};

pub struct FeedStream {
    ws: SplitStream<tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>>,
}

impl FeedStream {
    pub async fn connect(roomid: u32) -> Self {
        let hosts_info = HostsInfo::call(roomid).await.unwrap();
        let host = &hosts_info.host_list.choose(&mut rng()).unwrap();
        let url = format!("wss://{}:{}/sub", host.host, host.wss_port);

        let (socket, _) = connect_async(url.as_str()).await.unwrap();
        let (mut socket_sender, socket_receiver) = socket.split();

        let init = Message::Binary(Package::create_init_request(roomid, hosts_info.token).encode());
        socket_sender.send(init).await.unwrap();

        spawn(async move {
            let heartbeat = Message::Binary(Package::HeartbeatRequest().encode());
            let mut interval = time::interval(Duration::from_secs(HEARTBEAT_RATE_SEC));
            loop {
                interval.tick().await;
                socket_sender.send(heartbeat.clone()).await.unwrap();
            }
        });

        Self {
            ws: socket_receiver,
        }
    }
}

impl Stream for FeedStream {
    type Item = Vec<u8>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let p = ready!(Pin::new(&mut self.ws).poll_next(cx));

        Poll::Ready(match p {
            Some(Ok(Message::Binary(payload))) => Some(payload),
            Some(Ok(_)) => unreachable!(),
            Some(Err(error)) => {
                eprintln!("FEEDSTREAM! {}", error);
                None
            }
            None => None,
        })
    }
}
