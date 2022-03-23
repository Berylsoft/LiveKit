pub mod config {
    pub const FEED_HEARTBEAT_RATE_SEC: u64 = 30;
    pub const FEED_INIT_INTERVAL_MS: u64 = 100;
    pub const FEED_RETRY_INTERVAL_MS: u64 = 5000;
    pub const FEED_INIT_RETRY_INTERVAL_SEC: u64 = 5;
    pub const FEED_TCP_BUFFER_SIZE: usize = 1024 * 8;
}

pub mod util {
    pub fn now() -> u64 {
        use std::time::{SystemTime, UNIX_EPOCH};
        SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis().try_into().unwrap()
    }
    
    pub fn crc32(raw: &[u8]) -> u32 {
        let mut hasher = crc32fast::Hasher::new();
        hasher.update(raw);
        hasher.finalize()
    }    
}

pub mod payload;
#[cfg(feature = "package")]
pub mod package;
#[cfg(feature = "stream")]
pub mod stream;
#[cfg(feature = "schema")]
pub mod schema;
