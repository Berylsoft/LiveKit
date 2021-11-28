pub mod config {
    pub const FEED_HEARTBEAT_RATE_SEC: u64 = 30;
    pub const FEED_INIT_INTERVAL_MILLISEC: u64 = 100;
    pub const FEED_RETRY_INTERVAL_MILLISEC: u64 = 5000;
    pub const FEED_INIT_RETRY_INTERVAL_SEC: u64 = 5000;
    pub const FEED_TCP_BUFFER_SIZE: usize = 4096;
}

pub mod util;
#[cfg(feature = "package")]
pub mod package;
#[cfg(feature = "stream")]
pub mod stream;
#[cfg(feature = "schema")]
pub mod schema;
#[cfg(feature = "client")]
pub mod storage;
#[cfg(feature = "client")]
pub mod client;
