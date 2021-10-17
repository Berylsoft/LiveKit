use std::sync::Arc;
use tokio::time::{sleep, Duration};
use futures::StreamExt;
use async_recursion::async_recursion;
use async_channel::{Sender, SendError};
use rocksdb::DB;
use crate::{
    config::FEED_RETRY_INTERVAL_MILLISEC,
    api::room::HostsInfo,
    util::{Timestamp, http::HttpClient},
    feed::{package::Package, stream::FeedStream},
};

#[derive(Clone, Debug)]
pub enum Event {
    Init,
    Open,
    Close,
    Popularity(u32),
    Message(String),
}

impl Package {
    #[async_recursion]
    async fn send_as_events(self, sender: &Sender<Event>) -> Result<(), SendError<Event>> {
        match self {
            Package::Json(payload) => {
                sender.send(Event::Message(payload)).await?;
            },
            Package::Multi(payloads) => {
                for payload in payloads {
                    payload.send_as_events(sender).await?
                }
            },
            Package::HeartbeatResponse(payload) => {
                sender.send(Event::Popularity(payload)).await?;
            },
            Package::InitResponse(_) => {},
            _ => unreachable!(),
        }
        Ok(())
    }
}

pub async fn client(roomid: u32, http_client: Arc<HttpClient>, sender: Sender<Event>, storage: DB) {
    loop {
        let hosts_info = HostsInfo::call(&http_client, roomid).await.unwrap();
        let stream = FeedStream::connect(roomid, hosts_info).await.unwrap();
        sender.send(Event::Open).await.unwrap();
        stream.for_each(|message| async {
            let message = message;
            storage.put(Timestamp::now().to_bytes(), &message).unwrap();
            Package::decode(&message).send_as_events(&sender).await.unwrap();
        }).await;
        sender.send(Event::Close).await.unwrap();
        sleep(Duration::from_millis(FEED_RETRY_INTERVAL_MILLISEC)).await;
    }
}

pub async fn client_rec(roomid: u32, http_client: Arc<HttpClient>, storage: DB) {
    loop {
        let hosts_info = HostsInfo::call(&http_client, roomid).await.unwrap();
        let stream = FeedStream::connect(roomid, hosts_info).await.unwrap();
        eprintln!("[{:010}]open", roomid);
        stream.for_each(|message| async {
            let message = message;
            storage.put(Timestamp::now().to_bytes(), &message).unwrap();
            eprintln!("[{:010}]recv {}", roomid, message.len());
        }).await;
        eprintln!("[{:010}]close", roomid);
        sleep(Duration::from_millis(FEED_RETRY_INTERVAL_MILLISEC)).await;
    }
}

pub fn open_storage(path: String) -> Result<DB, rocksdb::Error> {
    // reserved independent function to contain tuning configurations later
    DB::open_default(path)
}
