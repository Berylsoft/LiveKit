use std::{io::Write, fs::OpenOptions};
use rand::{Rng, thread_rng as rng};
use tokio::spawn;
use futures::Future;
use async_channel::{unbounded as channel, Receiver};
use livekit_api::{client::HttpClient, info::{RoomInfo, UserInfo}};
use livekit_feed_client::{storage::sled::Db, schema::Event, client::client_sender};
use crate::config::*;

pub struct Room {
    roomid: u32,
    info: RoomInfo,
    user_info: UserInfo,
    receiver: Receiver<Event>,
    config: Config,
    http_client: HttpClient,
}

impl Room {
    pub async fn init(sroomid: u32, config: &Config, db: &Db, http_client: HttpClient, http_client2: HttpClient) -> Self {
        let info = RoomInfo::call(&http_client, sroomid).await.unwrap();
        let roomid = info.room_id;
        let user_info = UserInfo::call(&http_client, roomid).await.unwrap();
        let (sender, receiver) = channel();
        let storage = db.open_tree(roomid.to_string()).unwrap();
        spawn(client_sender(roomid, http_client2, storage, sender));

        Room {
            roomid,
            info,
            user_info,
            receiver,
            config: config.clone(),
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
            match &config.file_template {
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

    pub async fn dump(&self) -> impl Future<Output = ()> {
        let config = self.config.dump.as_ref().unwrap();
        let debug = if let Some(true) = config.debug { true } else { false };
        let receiver = self.subscribe();
        let mut file = OpenOptions::new().write(true).create(true).append(true)
            .open(format!("{}/{}.txt", config.path, self.id())).unwrap();
        async move {
            while let Ok(message) = receiver.recv().await {
                if debug {
                    write!(file, "{:?}", message).unwrap();
                } else {
                    serde_json::to_writer(&mut file, &message).unwrap();
                }
                writeln!(file).unwrap();
            }
        }
    }

    pub fn record(&self) -> impl Future<Output = ()> {
        use livekit_stream::{flv};
        let config = self.config.record.as_ref().unwrap();
        match config.mode {
            RecordMode::FlvRaw => flv::download(self.http_client.clone(), self.id(), self.record_file_path()),
            _ => unimplemented!(),
        }
    }
}
