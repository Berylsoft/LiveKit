use std::{pin::Pin, task::{Context, Poll}};
use tokio::{
    spawn,
    time::{self, Duration},
    sync::mpsc::{self, error::TryRecvError},
    io::{self, AsyncReadExt, AsyncWriteExt},
    net::{TcpStream, tcp::{OwnedReadHalf}},
};
use futures::{Stream, StreamExt, SinkExt, ready};
use rand::{seq::SliceRandom, thread_rng as rng};
use crate::{
    config::FEED_HEARTBEAT_RATE_SEC,
    api::room::HostsInfo,
    feed::package::Package,
};

pub struct FeedStream {
    roomid: u32,
    tcp: OwnedReadHalf,
    error: mpsc::Receiver<io::Error>,
}

impl FeedStream {
    pub async fn connect(roomid: u32, hosts_info: HostsInfo) -> Result<Self, io::Error> {
        let (error_sender, error_receiver) = mpsc::channel::<io::Error>(2);

        let host = &hosts_info.host_list.choose(&mut rng()).unwrap();
        let tcp = TcpStream::connect((host.host.as_str(), host.port)).await?;
        let (receiver, mut sender) = tcp.into_split();

        let init = Package::create_init_request(roomid, hosts_info.token).encode();
        sender.write_all(init.as_slice()).await?;

        spawn(async move {
            let heartbeat = Package::HeartbeatRequest().encode();
            let mut interval = time::interval(Duration::from_secs(FEED_HEARTBEAT_RATE_SEC));
            loop {
                interval.tick().await;
                if let Err(error) = sender.write_all(heartbeat.as_slice()).await {
                    if let Err(_) = error_sender.send(error).await {
                        break
                    }
                }
            }
        });

        Ok(Self {
            roomid,
            tcp: receiver,
            error: error_receiver,
        })
    }
}

impl Stream for FeedStream {
    type Item = Vec<u8>;

    // unfinished
    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let message = ready!(Pin::new(&mut self.tcp).poll_peek(cx));
        let heartbeat_error = self.error.try_recv();

        Poll::Ready(match heartbeat_error {
            Ok(error) => {
                log::warn!("[{:010} FEEDSTREAM HEARTBEAT]! {}", self.roomid, error);
                None
            },
            Err(TryRecvError::Empty) => match message {
                Some(Ok(payload)) => Some(payload),
                Some(Err(error)) => {
                    log::warn!("[{:010} FEEDSTREAM]! {}", self.roomid, error);
                    None
                },
                None => None,
            },
            Err(TryRecvError::Disconnected) => unreachable!(),
        })
    }
}
