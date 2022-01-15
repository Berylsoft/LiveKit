use rand::{Rng, thread_rng as rng};
use tokio::{spawn, fs};
use async_channel::{Sender, Receiver};
use livekit_api::{client::{HttpClient, RestApiResult}, info::{RoomInfo, UserInfo, GetRoomInfo, GetUserInfo}};
use livekit_feed::schema::Event as FeedEvent;
use livekit_feed_storage::{Db, open_db};
use crate::config::*;

macro_rules! template {
    ($template:expr, $($k:expr => $v:expr),*, $(,)?) => {
        $template
        $(
            .replace($k, $v.as_str())
        )*
    };
}

pub enum Event {
    Feed(FeedEvent),
}

pub struct Group {
    pub(crate) config: Config,
    pub(crate) http_client: HttpClient,
    pub(crate) db: Db,
}

impl Group {
    pub async fn init(config: Config) -> Group {
        if let Some(dump_config) = &config.dump {
            fs::create_dir_all(&dump_config.path).await.expect("creating dump directory error");
        }
        if let Some(record_config) = &config.record {
            fs::create_dir_all(&record_config.path).await.expect("creating record directory error");
        }

        Group {
            http_client: match config.http.clone() {
                Some(HttpConfig { access, proxy }) => HttpClient::new(access, proxy),
                None => HttpClient::new_bare(),
            },
            db: open_db(&config.storage.path).expect("opening storage error"),
            config,
        }
    }

    pub async fn spawn(&self, msroomid: i64) {
        if msroomid >= 0 {
            let sroomid = msroomid.try_into().unwrap();
            Room::init(sroomid, self).await.expect("fetching room status error");
        }
    }
}

pub struct Room {
    pub(crate) roomid: u32,
    pub(crate) info: RoomInfo,
    pub(crate) user_info: UserInfo,
    pub(crate) config: Config,
    pub(crate) http_client: HttpClient,
    pub(crate) tx: Sender<Event>,
    pub(crate) rx: Receiver<Event>,
}

impl Room {
    pub async fn init(sroomid: u32, group: &Group) -> RestApiResult<()> {
        let http_client = group.http_client.clone();
        let info = http_client.call(&GetRoomInfo{ sroomid }).await?;
        let roomid = info.room_id;
        let user_info = http_client.call(&GetUserInfo{ roomid }).await?;
        let (tx, rx) = async_channel::unbounded();

        let _self = Room {
            roomid: info.room_id,
            info,
            user_info,
            config: group.config.clone(),
            http_client,
            tx,
            rx,
        };

        spawn(_self.feed_client(group).await);

        if let Some(t) = _self.dump().await {
            spawn(t);
        }

        if let Some(t) = _self.simple_record().await {
            spawn(t);
        }

        // Ok(_self)
        Ok(())
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
        self.info = self.http_client.call(&GetRoomInfo{ sroomid: self.roomid }).await?;
        self.user_info = self.http_client.call(&GetUserInfo{ roomid: self.roomid }).await?;
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
}

