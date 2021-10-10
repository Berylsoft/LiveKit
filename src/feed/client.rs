use tokio::{spawn, sync::broadcast::{channel, Sender, Receiver, error::SendError}, time::{sleep, Duration}};
use futures::{future, StreamExt};
use rocksdb::DB;
use crate::{
    config::{RETRY_INTERVAL_SEC, EVENT_CHANNEL_BUFFER_SIZE},
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

impl Package {
    fn send_as_events(self, sender: &Sender<Event>) -> Result<(), SendError<Event>> {
        match self {
            Package::Json(payload) => {
                sender.send(Event::Message(payload))?;
            },
            Package::Multi(payloads) => {
                for payload in payloads {
                    payload.send_as_events(sender)?
                }
            },
            Package::HeartbeatResponse(payload) => {
                sender.send(Event::Popularity(payload))?;
            },
            Package::InitResponse(_) => {},
            _ => unreachable!(),
        }
        Ok(())
    }
}

pub async fn client(roomid: u32, sender: Sender<Event>, storage: DB) {
    loop {
        let stream = FeedStream::connect(roomid).await;
        sender.send(Event::Open).unwrap();
        stream.for_each(|message| {
            storage.put(Timestamp::now().to_bytes(), &message).unwrap();
            Package::decode(&message).send_as_events(&sender).unwrap();
            future::ready(())
        }).await;
        sender.send(Event::Close).unwrap();
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

pub fn open_storage(path: String) -> DB {
    DB::open_default(path).unwrap()
}

pub fn init(roomid: u32, storage_path: String) -> Receiver<Event> {
    let (sender, receiver) = channel(EVENT_CHANNEL_BUFFER_SIZE);
    let storage = open_storage(storage_path);
    spawn(client(roomid, sender, storage));
    receiver
}

pub fn init_record_only(roomid: u32, storage_path: String) {
    let storage = open_storage(storage_path);
    spawn(client_record_only(roomid, storage));
}
