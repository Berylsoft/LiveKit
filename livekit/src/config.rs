use serde::{Serialize, Deserialize};

pub const VERSION: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"), "-alpha-ms2");
pub const STORAGE_VERSION: &str = "alpha2";

pub const EVENT_CHANNEL_BUFFER_SIZE: usize = 1024;

pub const FEED_HEARTBEAT_RATE_SEC: u64 = 30;
pub const FEED_INIT_INTERVAL_MILLISEC: u64 = 100;
pub const FEED_RETRY_INTERVAL_SEC: u64 = 10;

pub const STREAM_DEFAULT_FILE_TEMPLATE: &str = "{roomid}-{date}-{time}{ms}-{title}";

#[derive(Serialize, Deserialize)]
pub struct General {
    pub config: GeneralConfig,
    pub groups: Vec<Group>,
}

#[derive(Serialize, Deserialize)]
pub struct GeneralConfig {
    pub rest_api_proxy: Option<String>,
    pub emulate_browser: bool,
}

#[derive(Serialize, Deserialize)]
pub struct Group {
    pub config: GroupConfig,
    pub rooms: Vec<RoomConfig>,
}

#[derive(Serialize, Deserialize)]
pub struct GroupRecordConfig {
    pub quality: Option<Vec<i32>>,
    pub file_root: String,
    pub file_template: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct GroupConfig {
    pub storage_root: String,
    pub access_token: Option<String>,
    pub record: GroupRecordConfig,
}

#[derive(Serialize, Deserialize)]
pub struct RoomConfig {
    pub roomid: u32,
    pub alias: Option<String>,
}