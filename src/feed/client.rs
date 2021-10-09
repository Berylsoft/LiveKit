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
pub type SendError = broadcast::error::SendError<Event>;
pub use broadcast::channel;

impl Package {
    fn send_as_events(self, channel_sender: &Sender) -> Result<(), SendError> {
        match self {
            Package::Json(payload) => {
                channel_sender.send(Event::Message(payload))?;
            },
            Package::Multi(payloads) => {
                for payload in payloads {
                    payload.send_as_events(channel_sender)?
                }
            },
            Package::HeartbeatResponse(payload) => {
                channel_sender.send(Event::Popularity(payload))?;
            },
            Package::InitResponse(_) => {},
            _ => unreachable!(),
        }
        Ok(())
    }
}

pub async fn client(roomid: u32, channel_sender: Sender, storage: DB) {
    loop {
        let stream = FeedStream::connect(roomid).await;
        channel_sender.send(Event::Open).unwrap();
        stream.for_each(|message| {
            storage.put(Timestamp::now().to_bytes(), &message).unwrap();
            Package::decode(&message).send_as_events(&channel_sender).unwrap();
            future::ready(())
        }).await;
        channel_sender.send(Event::Close).unwrap();
        sleep(Duration::from_secs(RETRY_INTERVAL_SEC)).await;
    }
}

pub async fn client_record_only(roomid: u32, storage: DB) {
    loop {
        let stream = FeedStream::connect(roomid).await;
        stream.for_each(|message| {
            storage.put(Timestamp::now().to_bytes(), &message).unwrap();
            future::ready(())
        }).await;
        sleep(Duration::from_secs(RETRY_INTERVAL_SEC)).await;
    }
}
