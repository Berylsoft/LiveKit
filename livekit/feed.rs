use tokio::time::{sleep, Duration};
use futures::{Future, StreamExt};
use livekit_api::feed::GetHostsInfo;
use livekit_feed::{config::*, stream::FeedStream, schema::Event as FeedEvent, storage::{open_storage, insert_payload}};
use crate::room::{Room, Event, Group};

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

impl Room {
    pub async fn feed_client(&self, group: &Group) -> impl Future<Output = ()> {
        let roomid = self.roomid;
        let storage = open_storage(&group.db, roomid);
        let http_client = group.http_client.clone();
        let tx = self.tx.clone();

        async move {
            loop {
                let hosts_info = unwrap_or_continue!(
                    http_client.call(&GetHostsInfo { roomid }).await,
                    |err| log::warn!("[{: >10}] get hosts error {:?}", roomid, err)
                );

                let mut stream = unwrap_or_continue!(
                    FeedStream::connect_ws(roomid, hosts_info).await,
                    |err| log::warn!("[{: >10}] error during connecting {:?}", roomid, err)
                );

                log::info!("[{: >10}] open", roomid);

                while let Some(may_payload) = stream.next().await {
                    if let Some(payload) = may_payload {
                        insert_payload(&storage, &payload).await;
                        for event in FeedEvent::from_raw(payload.payload) {
                            tx.send(Event::Feed(event)).await.unwrap();
                        }
                    }
                }

                log::info!("[{: >10}] close", roomid);

                sleep(Duration::from_millis(FEED_RETRY_INTERVAL_MS)).await;
            }
        }
    }
}
