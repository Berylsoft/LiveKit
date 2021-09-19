use serde::{Deserialize, Serialize};

pub const VERSION: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"), "-alpha-ms2");
pub const STORAGE_VERSION: &str = "alpha1";

pub const EVENT_CHANNEL_BUFFER_SIZE: usize = 32;
pub const HEARTBEAT_RATE_SEC: u64 = 30;

#[derive(Serialize, Deserialize)]
pub struct Config {
    pub rooms: Vec<Room>,
    pub storage_root: String,
}

#[derive(Serialize, Deserialize)]
pub struct Room {
    pub roomid: u32,
    pub alias: Option<String>,
}

impl Room {
    pub fn storage_name(&self) -> String {
        // TODO actual roomid
        match &self.alias {
            None => format!("{}-{}", self.roomid, STORAGE_VERSION),
            Some(alias) => format!("{}-{}", alias, STORAGE_VERSION),
        }
    }
}
