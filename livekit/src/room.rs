use rand::{Rng, thread_rng as rng};
use tokio::spawn;
use futures::Future;
use async_channel::{unbounded as channel, Receiver};
use sled::Db;
use crate::{
    config::{
        STREAM_DEFAULT_FILE_TEMPLATE,
        Config, RecordMode,
    },
    util::http::HttpClient,
    api::room::{RoomInfo, UserInfo},
    feed::client::{Event, client},
};

pub struct Room {
    roomid: u32,
    info: RoomInfo,
    user_info: UserInfo,
    receiver: Receiver<Event>,
    config: Config,
    http_client: HttpClient,
}

impl Room {
    pub async fn init(sroomid: u32, config: Config, db: &Db, http_client: HttpClient, http_client2: HttpClient) -> Self {
        let info = RoomInfo::call(&http_client, sroomid).await.unwrap();
        let roomid = info.room_id;
        let user_info = UserInfo::call(&http_client, roomid).await.unwrap();
        let (sender, receiver) = channel();
        let storage = db.open_tree(roomid.to_string()).unwrap();
        spawn(client(roomid, http_client2, sender, storage));

        Room {
            roomid,
            info,
            user_info,
            receiver,
            config,
            http_client,
        }
    }

    pub fn id(&self) -> u32 {
        self.roomid
    }

    pub fn config(&self) -> Config {
        self.config.clone()
    }

    pub fn info(&self) -> RoomInfo {
        self.info.clone()
    }

    pub fn user_info(&self) -> UserInfo {
        self.user_info.clone()
    }

    pub async fn update_info(&mut self) {
        self.info = RoomInfo::call(&self.http_client, self.roomid).await.unwrap();
        self.user_info = UserInfo::call(&self.http_client, self.roomid).await.unwrap();
    }

    pub fn record_file_path(&self) -> String {
        let config = self.config.record.as_ref().unwrap();
        let template = format!(
            "{}/{}.flv",
            config.file_root,
            match config.file_template.as_ref() {
                None => STREAM_DEFAULT_FILE_TEMPLATE,
                Some(template) => template.as_str(),
            },
        );
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
            .replace("{name}", self.user_info.info.uname.as_str())
            .replace("{parea}", self.info.parent_area_name.as_str())
            .replace("{area}", self.info.area_name.as_str())
    }

    pub fn subscribe(&self) -> Receiver<Event> {
        self.receiver.clone()
    }

    pub fn print_events_to_stdout(&self) -> impl Future<Output = ()> {
        let receiver = self.subscribe();
        let roomid = self.id();
        async move {
            while let Ok(message) = receiver.recv().await {
                println!("[{: >10}] {:?}", roomid, message);
            }
        }
    }

    pub fn record(&self) -> Option<impl Future<Output = ()>> {
        use crate::stream::{flv};
        match self.config.record.as_ref() {
            None => None,
            Some(config) => Some(match config.mode {
                RecordMode::FlvRaw => flv::download(self.http_client.clone(), self.id(), self.record_file_path()),
                _ => unimplemented!(),
            })
        }
    }
}
