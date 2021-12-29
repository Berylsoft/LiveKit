use tokio::time::{sleep, Duration};
use futures::StreamExt;
// #[cfg(feature = "client_sender_only")]
use async_channel::Sender;
use livekit_api::{client::HttpClient, feed::HostsInfo};
// #[cfg(feature = "client_rec")]
use livekit_feed::{config::*, stream::FeedStream, schema::Event};
use livekit_feed_storage::{sled::Tree, insert_payload};

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
        sleep(Duration::from_millis(FEED_RETRY_INTERVAL_MS)).await;
    }};
}

// #[cfg(feature = "client_rec")]
pub async fn client_rec(roomid: u32, http_client: HttpClient, storage: Tree) {
    loop {
        let mut stream = connect_impl!(roomid, http_client);
        while let Some(payload) = stream.next().await {
            insert_payload(&storage, &payload).unwrap();
        }
        close_impl!(roomid);
    }
}

// #[cfg(feature = "client_sender")]
pub async fn client_sender(roomid: u32, http_client: HttpClient, storage: Tree, sender: Sender<Event>) {
    loop {
        let mut stream = connect_impl!(roomid, http_client);
        while let Some(payload) = stream.next().await {
            insert_payload(&storage, &payload).unwrap();
            for event in Event::from_raw(payload.payload) {
                sender.send(event).await.unwrap();
            }
        }
        close_impl!(roomid);
    }
}

// #[cfg(feature = "client_sender_only")]
pub async fn client_sender_only(roomid: u32, http_client: HttpClient, sender: Sender<Event>) {
    loop {
        let mut stream = connect_impl!(roomid, http_client);
        while let Some(payload) = stream.next().await {
            for event in Event::from_raw(payload.payload) {
                sender.send(event).await.unwrap();
            }
        }
        close_impl!(roomid);
    }
}
