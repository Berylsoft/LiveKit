use std::path::PathBuf;
use serde::{Serialize, Deserialize};
use livekit_api::client::{Access, HttpClient};
use livekit_feed::transfer::OutputKind;

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
#[serde(tag = "type")]
pub enum RecordFragmentMode {
    ByTime { per_min: u32 },
    BySize { per_mb: u32 },
}

#[derive(Serialize, Deserialize, Clone)]
pub struct RecordConfig {
    pub mode: RecordMode,
    pub qn: Option<Vec<i32>>,
    pub path: PathBuf,
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
    pub path: PathBuf,
    pub kind: OutputKind,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct StorageConfig {
    pub path: PathBuf,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Config {
    pub storage: StorageConfig,
    pub http: Option<HttpConfig>,
    pub dump: Option<DumpConfig>,
    pub record: Option<RecordConfig>,
}
