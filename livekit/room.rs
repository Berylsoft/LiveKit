use std::fs::OpenOptions;
use rand::{Rng, thread_rng as rng};
use tokio::spawn;
use futures::Future;
use async_channel::{unbounded as channel, Receiver};
use livekit_api::{client::{HttpClient, RestApiResult}, info::{RoomInfo, UserInfo}};
use livekit_feed::{payload::Payload, schema::Event};
use livekit_feed_storage::{Db, open_storage};
use crate::{config::*, feed::client_sender, transfer::write};

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
    receiver: Receiver<Payload>,
    config: Config,
    http_client: HttpClient,
}

impl Room {
    pub async fn init(sroomid: u32, config: &Config, db: &Db, http_client: HttpClient, http_client2: HttpClient) -> RestApiResult<Self> {
        let info = RoomInfo::call(&http_client, sroomid).await?;
        let roomid = info.room_id;
        let user_info = UserInfo::call(&http_client, roomid).await?;
        let (sender, receiver) = channel();
        let storage = open_storage(&db, roomid).unwrap();
        spawn(client_sender(roomid, http_client2, storage, sender));

        Ok(Room {
            roomid,
            info,
            user_info,
            receiver,
            config: config.clone(),
            http_client,
        })
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

    pub async fn update_info(&mut self) -> RestApiResult<()> {
        self.info = RoomInfo::call(&self.http_client, self.roomid).await?;
        self.user_info = UserInfo::call(&self.http_client, self.roomid).await?;
        Ok(())
    }

    pub fn record_file_name(&self) -> String {
        let config = self.config.record.as_ref().unwrap();
        let template = match &config.name_template {
            None => STREAM_DEFAULT_FILE_TEMPLATE,
            Some(template) => template.as_str(),
        };
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

    pub fn subscribe(&self) -> Receiver<Payload> {
        self.receiver.clone()
    }

    pub async fn dump(&self) -> impl Future<Output = ()> {
        let config = self.config.dump.as_ref().unwrap();
        let kind = config.kind.clone();
        let receiver = self.subscribe();
        let mut file = OpenOptions::new().write(true).create(true).append(true).open({
            let mut path = config.path.clone();
            path.push(format!("{}.txt", self.id()));
            path
        }).expect("opening dump file error");
        async move {
            while let Ok(payload) = receiver.recv().await {
                for event in Event::from_raw(payload.payload) {
                    write(&mut file, &kind, &event);
                }
            }
        }
    }

    pub fn record(&self) -> impl Future<Output = ()> {
        let config = self.config.record.as_ref().unwrap();
        match config.mode {
            RecordMode::FlvRaw => crate::flv::download(self.http_client.clone(), self.id(), {
                let mut path = config.path.clone();
                path.push(format!("{}.flv", self.record_file_name()));
                path
            }),
            _ => unimplemented!(),
        }
    }
}
