use tokio::time::{sleep, Duration};
use futures::StreamExt;
// #[cfg(feature = "client_sender_only")]
use async_channel::Sender;
use livekit_api::{client::HttpClient, feed::HostsInfo};
// #[cfg(feature = "client_rec")]
use crate::storage::sled::Tree;
use crate::{
    config::{FEED_RETRY_INTERVAL_MILLISEC, FEED_INIT_RETRY_INTERVAL_SEC},
    util::Timestamp,
    stream::FeedStream, package::Package, schema::Event,
};

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

macro_rules! connect_impl {
    ($roomid:expr, $http_client:expr) => {{
        let hosts_info = unwrap_or_continue!(
            HostsInfo::call(&$http_client, $roomid).await,
            |err| log::warn!("[{: >10}] get hosts error {:?}", $roomid, err)
        );
        let stream = unwrap_or_continue!(
            FeedStream::connect_ws($roomid, hosts_info).await,
            |err| log::warn!("[{: >10}] error during connecting {:?}", $roomid, err)
        );
        log::info!("[{: >10}] open", $roomid);
        stream
    }};
}

macro_rules! close_impl {
    ($roomid:expr) => {{
        log::info!("[{: >10}] close", $roomid);
        sleep(Duration::from_millis(FEED_RETRY_INTERVAL_MILLISEC)).await;
    }};
}

// #[cfg(feature = "client_rec")]
pub async fn client_rec(roomid: u32, http_client: HttpClient, storage: Tree) {
    loop {
        let mut stream = connect_impl!(roomid, http_client);
        while let Some(message) = stream.next().await {
            storage.insert(Timestamp::now().to_bytes(), message.as_slice()).unwrap();
        }
        close_impl!(roomid);
    }
}

// #[cfg(feature = "client_sender")]
pub async fn client_sender(roomid: u32, http_client: HttpClient, storage: Tree, sender: Sender<Event>) {
    loop {
        let mut stream = connect_impl!(roomid, http_client);
        while let Some(message) = stream.next().await {
            storage.insert(Timestamp::now().to_bytes(), message.as_slice()).unwrap();
            for package in Package::decode(message).flatten() {
                sender.send(Event::from_package(package)).await.unwrap();
            }
        }
        close_impl!(roomid);
    }
}

// #[cfg(feature = "client_sender_only")]
pub async fn client_sender_only(roomid: u32, http_client: HttpClient, sender: Sender<Event>) {
    loop {
        let mut stream = connect_impl!(roomid, http_client);
        while let Some(message) = stream.next().await {
            for package in Package::decode(message).flatten() {
                sender.send(Event::from_package(package)).await.unwrap();
            }
        }
        close_impl!(roomid);
    }
}
