use tokio::time::{sleep, Duration};
use futures::StreamExt;
use async_recursion::async_recursion;
use async_channel::{Sender, SendError};
use rocksdb::DB;
use crate::{
    config::FEED_RETRY_INTERVAL_MILLISEC,
    api::room::HostsInfo,
    util::{Timestamp, http::HttpClient},
    feed::{package::Package, stream::FeedStream, schema::*},
};

#[derive(Debug)]
pub enum Event {
    Init,
    Open,
    Close,
    Popularity(u32),
    Unknown(String),
    Danmaku(Danmaku),
}

pub fn dispatcher(payload: &str) -> Option<Event> {
    let raw: serde_json::Value = serde_json::from_str(payload).ok()?;
    match raw["cmd"].as_str()? {
        "DANMU_MSG" => Some(Event::Danmaku(Danmaku::new(&raw["info"]).ok()?)),
        _ => None,
    }
}

impl Package {
    #[async_recursion]
    async fn send_as_events(self, sender: &Sender<Event>) -> Result<(), SendError<Event>> {
        match self {
            Package::Json(payload) => {
                sender.send(dispatcher(payload.as_str()).unwrap_or_else(|| Event::Unknown(payload))).await?;
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

pub async fn client(roomid: u32, http_client: HttpClient, sender: Sender<Event>, storage: DB) {
    loop {
        let hosts_info = HostsInfo::call(&http_client, roomid).await.unwrap();
        let mut stream = FeedStream::connect_ws(roomid, hosts_info).await.unwrap();
        sender.send(Event::Open).await.unwrap();
        while let Some(message) = stream.next().await {
            storage.put(Timestamp::now().to_bytes(), &message).unwrap();
            Package::decode(&message).send_as_events(&sender).await.unwrap();
        }
        sender.send(Event::Close).await.unwrap();
        sleep(Duration::from_millis(FEED_RETRY_INTERVAL_MILLISEC)).await;
    }
}

pub async fn client_rec(roomid: u32, http_client: HttpClient, storage: DB) {
    loop {
        let hosts_info = HostsInfo::call(&http_client, roomid).await.unwrap();
        let mut stream = FeedStream::connect_ws(roomid, hosts_info).await.unwrap();
        log::info!("[{: >10}] open", roomid);
        while let Some(message) = stream.next().await {
            storage.put(Timestamp::now().to_bytes(), message).unwrap();
        }
        log::info!("[{: >10}] close", roomid);
        sleep(Duration::from_millis(FEED_RETRY_INTERVAL_MILLISEC)).await;
    }
}

pub fn open_storage(path: String) -> Result<DB, rocksdb::Error> {
    // reserved independent function to contain tuning configurations later
    DB::open_default(path)
}
