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
    group_config: GroupConfig,
}

impl Room {
    pub async fn init(room_config: RoomConfig, group_config: GroupConfig) -> Self {
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
            group_config,
        }
    }

    pub fn id(&self) -> u32 {
        self.roomid
    }

    pub fn id_pad_string(&self) -> String {
        format!("[{:010}]", self.roomid)
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
        let time = chrono::Local::now();

        template
            .replace("{date}", time.format("%Y%m%d").to_string().as_str())
            .replace("{time}", time.format("%H%M%S").to_string().as_str())
            .replace("{ms}", time.format("%3f").to_string().as_str())
            .replace("{iso8601}", time.to_rfc3339_opts(chrono::SecondsFormat::Millis, false).to_string().as_str())
            .replace("{ts}", time.timestamp_millis().to_string().as_str())
            .replace("{random}", rng().gen_range(0..100).to_string().as_str())
            .replace("{roomid}", self.roomid.to_string().as_str())
            .replace("{title}", self.info.title.as_str())
            .replace("{name}", self.user_info.info.uname.to_string().as_str())
            .replace("{parea}", self.info.parent_area_name.as_str())
            .replace("{area}", self.info.area_name.as_str())
    }

    pub fn subscribe(&self) -> Receiver<Event> {
        self.receiver.clone()
    }

    pub fn print_events_to_stdout(&self) -> impl Future<Output = ()> {
        let receiver = self.subscribe();
        let roomid = self.id_pad_string();
        async move {
            loop {
                println!("{}{:?}", roomid, receiver.recv().await.unwrap());
            }
        }
    }

    pub fn download_stream_flv(&self) -> impl Future<Output = ()> {
        crate::stream::flv::download(self.id(), format!(
            "{}/{}",
            self.group_config.record.file_root,
            self.record_file_name(self.group_config.record.file_template.clone()),
        ))
    }
}
