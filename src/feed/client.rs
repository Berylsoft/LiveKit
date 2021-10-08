use tokio::{sync::broadcast, time::{sleep, Duration}};
use futures::{future, StreamExt};
use rocksdb::DB;
use crate::{
    config::RETRY_INTERVAL_SEC,
    util::Timestamp,
    feed::{package::Package, stream::FeedStream},
};

#[derive(Clone, Debug)]
pub enum Event {
    Open,
    Close,
    Popularity(u32),
    Message(String),
}

pub type Sender = broadcast::Sender<Event>;
pub type Receiver = broadcast::Receiver<Event>;
pub use broadcast::channel;

impl Package {
    pub fn send_as_events(self, channel_sender: &mut Sender) {
        // TODO process recursive `Multi` & return iter
        match self {
            Package::Multi(payloads) => for payload in payloads {
                match payload {
                    Package::Json(payload) => { channel_sender.send(Event::Message(payload)).unwrap(); },
                    _ => unreachable!(),
                }
            },
            Package::Json(payload) => { channel_sender.send(Event::Message(payload)).unwrap(); },
            Package::HeartbeatResponse(payload) => { channel_sender.send(Event::Popularity(payload)).unwrap(); },
            Package::InitResponse(_) => (),
            _ => unreachable!(),
        }
    }
}

pub async fn client(roomid: u32, mut channel_sender: Sender, storage: DB) {
    loop {
        let stream = FeedStream::connect(roomid).await;
        channel_sender.send(Event::Open).unwrap();
        stream.for_each(|message| {
            storage.put(Timestamp::now().to_bytes(), &message).unwrap();
            Package::decode(&message).send_as_events(&mut channel_sender);
            future::ready(())
        }).await;
        channel_sender.send(Event::Close).unwrap();
        sleep(Duration::from_secs(RETRY_INTERVAL_SEC)).await;
    }
}
