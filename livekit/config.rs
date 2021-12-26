use serde::{Serialize, Deserialize};
use livekit_api::client::{Access, HttpClient};

pub const ROOM_INFO_UPDATE_INTERVAL_SEC: u64 = 600;

pub const STREAM_DEFAULT_FILE_TEMPLATE: &str = "{roomid}-{date}-{time}{ms}-{title}";

#[derive(Serialize, Deserialize, Clone)]
pub struct Groups {
    pub group: Vec<Group>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Group {
    pub config: Config,
    pub rooms: Vec<i64>,
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
    pub path: String,
    pub name_template: Option<String>,
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
