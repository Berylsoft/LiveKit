use std::path::PathBuf;
use serde::{Serialize, Deserialize};
use livekit_api::client::Access;

pub const ROOM_INIT_INTERVAL_MS: u64 = 100;
pub const ROOM_INFO_UPDATE_INTERVAL_SEC: u64 = 600;

pub const STREAM_DEFAULT_FILE_TEMPLATE: &str = "{roomid}-{date}-{time}{ms}-{title}";

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct GlobalConfig {
    pub group: Vec<GroupConfig>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct GroupConfig {
    pub config: Config,
    pub rooms: Vec<i64>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Config {
    pub storage: StorageConfig,
    pub http: Option<HttpConfig>,
    pub dump: Option<DumpConfig>,
    pub record: Option<RecordConfig>,
}

// region: record

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum RecordMode {
    FlvRaw,
    FlvRawWithProperEnd,
    FlvReformed,
    FlvReformedIndexed,
    HlsRawSlices,
    HlsRawConcated,
    HlsReformed, // may or cannot
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(tag = "type")]
pub enum RecordFragmentMode {
    ByTime { per_min: u32 },
    BySize { per_mb: u32 },
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct RecordConfig {
    pub mode: RecordMode,
    pub qn: Option<Vec<i32>>,
    pub path: PathBuf,
    pub name_template: Option<String>,
    pub fragment: Option<RecordFragmentMode>,
}

// endregion

// region: http

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct HttpConfig {
    pub access: Option<Access>,
    pub proxy: Option<String>,
}

// endregion

// region: dump

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum DumpKind {
    Debug,
    NdJson,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct DumpConfig {
    pub path: PathBuf,
    pub kind: DumpKind,
}

// endregion

// region: storage

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct StorageConfig {
    pub path: PathBuf,
}

// endregion
