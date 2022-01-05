use tokio::time::{sleep, Duration};
use futures::StreamExt;
// #[cfg(feature = "client_sender_only")]
use async_channel::Sender;
use livekit_api::{client::HttpClient, feed::HostsInfo};
// #[cfg(feature = "client_rec")]
use livekit_feed::{config::*, payload::Payload, stream::FeedStream};
use livekit_feed_storage::{Tree, insert_payload};

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

pub async fn client_sender(roomid: u32, http_client: HttpClient, storage: Tree, sender: Sender<Payload>) {
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

        while let Some(may_payload) = stream.next().await {
            if let Some(payload) = may_payload {
                insert_payload(&storage, &payload);
                sender.send(payload).await.unwrap();
            }
        }

        log::info!("[{: >10}] close", roomid);

        sleep(Duration::from_millis(FEED_RETRY_INTERVAL_MS)).await;
    }
}
