use std::{io::Write, fs::OpenOptions};
use rand::{Rng, thread_rng as rng};
use tokio::spawn;
use futures::Future;
use async_channel::{unbounded as channel, Receiver};
use livekit_api::{client::HttpClient, info::{RoomInfo, UserInfo}};
use livekit_feed::schema::Event;
use livekit_feed_storage::{Db, open_storage};
use livekit_feed_client::thread::client_sender;
use crate::config::*;

macro_rules! template {
    ($template:expr, $($k:expr => $v:expr),*, $(,)?) => {
        $template
        $(
            .replace($k, $v.as_str())
        )*
    };
}

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
        let storage = open_storage(&db, roomid).unwrap();
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

    pub fn config(&self) -> &Config {
        &self.config
    }

    pub fn info(&self) -> &RoomInfo {
        &self.info
    }

    pub fn user_info(&self) -> &UserInfo {
        &self.user_info
    }

    pub async fn update_info(&mut self) {
        self.info = RoomInfo::call(&self.http_client, self.roomid).await.unwrap();
        self.user_info = UserInfo::call(&self.http_client, self.roomid).await.unwrap();
    }

    pub fn record_file_path(&self) -> String {
        let config = self.config.record.as_ref().unwrap();
        let template = format!(
            "{}/{}.flv",
            config.path,
            match &config.name_template {
                None => STREAM_DEFAULT_FILE_TEMPLATE,
                Some(template) => template.as_str(),
            },
        );
        let time = chrono::Local::now();

        template!(
            template,
            "{date}"    => time.format("%Y%m%d").to_string(),
            "{time}"    => time.format("%H%M%S").to_string(),
            "{ms}"      => time.format("%3f").to_string(),
            "{iso8601}" => time.to_rfc3339_opts(chrono::SecondsFormat::Millis, false).to_string(),
            "{ts}"      => time.timestamp_millis().to_string(),
            "{random}"  => format!("{:0>2}", rng().gen_range(0..100)),
            "{roomid}"  => self.roomid.to_string(),
            "{title}"   => self.info.title,
            "{name}"    => self.user_info.info.uname,
            "{parea}"   => self.info.parent_area_name,
            "{area}"    => self.info.area_name,
        )
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
            while let Ok(event) = receiver.recv().await {
                if debug {
                    write!(file, "{:?}", event).unwrap();
                } else {
                    if let Event::Unimplemented | Event::Ignored = event {
                        continue
                    }
                    serde_json::to_writer(&mut file, &event).unwrap();
                }
                writeln!(file).unwrap();
            }
        }
    }

    pub fn record(&self) -> impl Future<Output = ()> {
        use livekit_stream_get::{flv};
        let config = self.config.record.as_ref().unwrap();
        match config.mode {
            RecordMode::FlvRaw => flv::download(self.http_client.clone(), self.id(), self.record_file_path()),
            _ => unimplemented!(),
        }
    }
}
