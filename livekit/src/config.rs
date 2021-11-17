use serde::{Serialize, Deserialize};

pub const FEED_HEARTBEAT_RATE_SEC: u64 = 30;
pub const FEED_INIT_INTERVAL_MILLISEC: u64 = 100;
pub const FEED_RETRY_INTERVAL_MILLISEC: u64 = 5000;
pub const FEED_TCP_BUFFER_SIZE: usize = 4096;

pub const STREAM_RETRY_INTERVEL_MILLISEC: u64 = 6000;
pub const STREAM_CONNECT_TIMEOUT_MILLISEC: u64 = 5000;
pub const STREAM_NO_DATA_TIMEOUT_MILLISEC: u64 = 10000;

pub const ROOM_INFO_UPDATE_INTERVAL_SEC: u64 = 600;

pub const STREAM_DEFAULT_FILE_TEMPLATE: &str = "{roomid}-{date}-{time}{ms}-{title}";

pub const REFERER: &str = "https://live.bilibili.com/";
pub const API_HOST: &str = "https://api.live.bilibili.com";
pub const WEB_USER_AGENT: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/95.0.4638.69 Safari/537.36";

#[derive(Serialize, Deserialize, Clone)]
pub struct Group {
    pub config: Config,
    pub rooms: Vec<RoomConfig>,
}

#[derive(Serialize, Deserialize, Clone)]
pub enum RecordMode {
    FlvRaw,
    FlvRawWithProperEnd,
    FlvReformed,
    FlvReformedIndexed,
    HlsRawSlices,
    HlsRawConcated,
    HlsReformed, // may or cannot
}

#[derive(Serialize, Deserialize, Clone)]
pub enum RecordFragmentMode {
    ByTime(u32), // min
    BySize(u32), // MB
}

#[derive(Serialize, Deserialize, Clone)]
pub struct RecordConfig {
    pub mode: RecordMode,
    pub quality: Option<Vec<i32>>,
    pub file_root: String,
    pub file_template: Option<String>,
    pub fragment: Option<RecordFragmentMode>,
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
    pub record: Option<RecordConfig>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct RoomConfig {
    pub sroomid: u32,
    pub operational: bool,
}
