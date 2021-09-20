use serde::{Deserialize, Serialize};

pub const VERSION: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"), "-alpha-ms2");
pub const STORAGE_VERSION: &str = "alpha1";

pub const EVENT_CHANNEL_BUFFER_SIZE: usize = 32;
pub const HEARTBEAT_RATE_SEC: u64 = 30;

#[derive(Serialize, Deserialize)]
pub struct Config {
    pub general: GeneralConfig,
    pub rooms: Vec<RoomConfig>,
}

#[derive(Serialize, Deserialize)]
pub struct GeneralConfig {
    pub storage_root: String,
    pub record_root: String,
}

#[derive(Serialize, Deserialize)]
pub struct RoomConfig {
    pub roomid: u32,
    pub alias: Option<String>,
}
