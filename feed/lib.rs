pub mod config {
    pub const FEED_HEARTBEAT_RATE_SEC: u64 = 30;
    pub const FEED_INIT_INTERVAL_MS: u64 = 100;
    pub const FEED_RETRY_INTERVAL_MS: u64 = 5000;
    pub const FEED_INIT_RETRY_INTERVAL_SEC: u64 = 5000;
    pub const FEED_TCP_BUFFER_SIZE: usize = 4096;
    pub const FEED_STORAGE_CACHE_MAX_BYTE: u64 = 1024 * 1024 * 16;
    pub const FEED_STORAGE_FLUSH_INTERVAL_MS: u64 = 1000;
}

pub mod util;
#[cfg(feature = "package")]
pub mod package;
#[cfg(feature = "stream")]
pub mod stream;
#[cfg(feature = "schema")]
pub mod schema;
