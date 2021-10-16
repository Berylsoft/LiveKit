use rand::{Rng, thread_rng as rng};
use tokio::spawn;
use futures::Future;
use async_channel::{unbounded as channel, Receiver};
use crate::{
    config::{
        STORAGE_VERSION, STREAM_DEFAULT_FILE_TEMPLATE,
        RoomConfig, GroupConfig,
    },
    api::room::{RoomInfo, UserInfo},
    feed::client::{Event, open_storage, client},
};

pub struct Room {
    roomid: u32,
    info: RoomInfo,
    user_info: UserInfo,
    receiver: Receiver<Event>,
}

impl Room {
    pub async fn init(room_config: &RoomConfig, group_config: &GroupConfig) -> Self {
        let info = RoomInfo::call(room_config.roomid).await.unwrap();
        let roomid = info.room_id;
        let user_info = UserInfo::call(roomid).await.unwrap();
        let storage_name = format!(
            "{}-{}",
            match &room_config.alias {
                None => roomid.to_string(),
                Some(alias) => alias.clone(),
            },
            STORAGE_VERSION,
        );
        let (sender, receiver) = channel();
        let storage = open_storage(format!("{}/{}", group_config.storage_root, storage_name)).unwrap();
        spawn(client(roomid, sender, storage));

        Room {
            roomid,
            info,
            user_info,
            receiver,
        }
    }

    pub fn id(&self) -> u32 {
        self.roomid
    }

    pub fn info(&self) -> RoomInfo {
        self.info.clone()
    }

    pub async fn update_info(&mut self) {
        self.info = RoomInfo::call(self.roomid).await.unwrap();
    }

    pub fn user_info(&self) -> UserInfo {
        self.user_info.clone()
    }

    pub async fn update_user_info(&mut self) {
        self.user_info = UserInfo::call(self.roomid).await.unwrap();
    }

    pub fn record_file_name(&self, file_template: Option<String>) -> String {
        // group_config.record.file_template.clone()
        let template = format!("{}.flv", file_template.unwrap_or_else(|| STREAM_DEFAULT_FILE_TEMPLATE.to_string()));
        let time = chrono::Utc::now();

        template
            .replace("{date}", time.format("%y%M%d").to_string().as_str())
            .replace("{time}", time.format("%H%m%s").to_string().as_str())
            .replace("{ms}", time.format("%.3f").to_string().as_str())
            .replace("{ts}", time.timestamp_millis().to_string().as_str())
            .replace("{random}", rng().gen_range(0..100).to_string().as_str())
            .replace("{roomid}", self.roomid.to_string().as_str())
            .replace("{title}", self.info.title.as_str())
            .replace("{name}", self.user_info.info.uname.to_string().as_str())
            .replace("{parea}", self.info.parent_area_name.as_str())
            .replace("{area}", self.info.area_name.as_str())
    }

    #[inline]
    pub fn subscribe(&self) -> Receiver<Event> {
        self.receiver.clone()
    }

    pub fn print_events_to_stdout(&self) -> impl Future<Output = ()> {
        let receiver = self.subscribe();
        async move {
            loop {
                println!("{:?}", receiver.recv().await.unwrap());
            }
        }
    }
}
