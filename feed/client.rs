use tokio::time::{sleep, Duration};
use futures::StreamExt;
use async_channel::Sender;
use sled::Tree;
use livekit_api::{client::HttpClient, feed::HostsInfo};
use crate::{
    config::{FEED_RETRY_INTERVAL_MILLISEC, FEED_INIT_RETRY_INTERVAL_SEC},
    util::Timestamp,
    stream::FeedStream, package::Package, schema::Event,
};

#[derive(Debug)]
pub enum WrappedEvent {
    Init,
    Open,
    Close,
    Event(Event),
}

pub async fn client(roomid: u32, http_client: HttpClient, sender: Sender<WrappedEvent>, storage: Tree) {
    loop {
        let hosts_info = HostsInfo::call(&http_client, roomid).await.unwrap();
        let mut stream = FeedStream::connect_ws(roomid, hosts_info).await.unwrap();
        sender.send(WrappedEvent::Open).await.unwrap();
        while let Some(message) = stream.next().await {
            storage.insert(Timestamp::now().to_bytes(), message.as_slice()).unwrap();
            for package in Package::decode(message).flatten() {
                sender.send(WrappedEvent::Event(Event::from_package(package))).await.unwrap();
            }
        }
        sender.send(WrappedEvent::Close).await.unwrap();
        sleep(Duration::from_millis(FEED_RETRY_INTERVAL_MILLISEC)).await;
    }
}

macro_rules! unwrap_or_continue {
    ($res:expr, $or:expr) => {
        match $res {
            Ok(val) => val,
            Err(err) => {
                $or(err);
                sleep(Duration::from_secs(FEED_INIT_RETRY_INTERVAL_SEC)).await;
                continue;
            }
        }
    };
}

pub async fn client_rec(roomid: u32, http_client: HttpClient, storage: Tree) {
    loop {
        let hosts_info = unwrap_or_continue!(
            HostsInfo::call(&http_client, roomid).await,
            |err| log::warn!("[{: >10}] get hosts error {:?}", roomid, err)
        );
        let mut stream = unwrap_or_continue!(
            FeedStream::connect_ws(roomid, hosts_info).await,
            |err| log::warn!("[{: >10}] error during connecting {:?}", roomid, err)
        );
        log::info!("[{: >10}] open", roomid);
        while let Some(message) = stream.next().await {
            storage.insert(Timestamp::now().to_bytes(), message.as_slice()).unwrap();
        }
        log::info!("[{: >10}] close", roomid);
        sleep(Duration::from_millis(FEED_RETRY_INTERVAL_MILLISEC)).await;
    }
}
