use serde::{Serialize, Deserialize};
use livekit_api::client::{Access, HttpClient};

pub const STREAM_RETRY_INTERVEL_MILLISEC: u64 = 6000;
pub const STREAM_CONNECT_TIMEOUT_MILLISEC: u64 = 5000;
pub const STREAM_NO_DATA_TIMEOUT_MILLISEC: u64 = 10000;

pub const ROOM_INFO_UPDATE_INTERVAL_SEC: u64 = 600;

pub const STREAM_DEFAULT_FILE_TEMPLATE: &str = "{roomid}-{date}-{time}{ms}-{title}";

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
pub struct HttpConfig {
    access: Option<Access>,
    proxy: Option<String>,
}

impl HttpConfig {
    pub async fn build(config: Option<HttpConfig>) -> HttpClient {
        match config {
            Some(HttpConfig { access, proxy }) => HttpClient::new(access, proxy).await,
            None => HttpClient::new(None, None).await,
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct DumpConfig {
    pub path: String,
    pub debug: Option<bool>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct StorageConfig {
    pub path: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Config {
    pub storage: StorageConfig,
    pub http: Option<HttpConfig>,
    pub dump: Option<DumpConfig>,
    pub record: Option<RecordConfig>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct RoomConfig {
    pub sroomid: Option<u32>,
    pub _sroomid: Option<u32>,
}

impl RoomConfig {
    pub fn unwrap(&self) -> Option<u32> {
        self.sroomid
    }

    pub fn decode(&self) -> Option<(u32, bool)> {
        match self {
            RoomConfig { sroomid: Some(sroomid), _sroomid: _ } => Some((*sroomid, true)),
            RoomConfig { sroomid: None, _sroomid: Some(sroomid) } => Some((*sroomid, false)),
            RoomConfig { sroomid: None, _sroomid: None } => None,
        }
    }

    pub fn encode(decoded: (u32, bool)) -> RoomConfig {
        if decoded.1 {
            RoomConfig {
                sroomid: Some(decoded.0),
                _sroomid: None,
            }
        } else {
            RoomConfig {
                sroomid: None,
                _sroomid: Some(decoded.0),
            }
        }
    }
}
