use serde::{Serialize, Deserialize};

pub const VERSION: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"), "-alpha-ms2");
pub const STORAGE_VERSION: &str = "alpha2";

pub const FEED_HEARTBEAT_RATE_SEC: u64 = 30;
pub const FEED_INIT_INTERVAL_MILLISEC: u64 = 100;
pub const FEED_RETRY_INTERVAL_MILLISEC: u64 = 5000;

pub const STREAMREC_RETRY_INTERVEL_MILLISEC: u64 = 6000;
pub const STREAMREC_CONNECT_TIMEOUT_MILLISEC: u64 = 5000;
pub const STREAMREC_NO_DATA_TIMEOUT_MILLISEC: u64 = 10000;

pub const ROOM_INFO_UPDATE_INTERVAL_SEC: u64 = 600;

pub const STREAMREC_DEFAULT_FILE_TEMPLATE: &str = "{roomid}-{date}-{time}{ms}-{title}";

#[derive(Serialize, Deserialize, Clone)]
pub struct Group {
    pub config: Config,
    pub rooms: Vec<RoomConfig>,
}

#[derive(Serialize, Deserialize, Clone)]
pub enum StreamrecMode {
    FlvRaw,
    FlvReformed,
    HlsRawSlices,
}

#[derive(Serialize, Deserialize, Clone)]
pub enum StreamrecFragmentMode {
    ByTime(u32), // min
    BySize(u32), // MB
}

#[derive(Serialize, Deserialize, Clone)]
pub struct StreamrecConfig {
    pub mode: StreamrecMode,
    pub quality: Option<Vec<i32>>,
    pub file_root: String,
    pub file_template: Option<String>,
    pub fragment: Option<StreamrecFragmentMode>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct CommonConfig {
    pub storage_root: String,
    pub access_token: Option<String>,
    pub api_proxy: Option<String>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Config {
    pub common: CommonConfig,
    pub streamrec: Option<StreamrecConfig>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct RoomConfig {
    pub sroomid: u32,
    pub operational: bool,
}
