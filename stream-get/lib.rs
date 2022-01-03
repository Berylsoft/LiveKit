pub mod config {
    pub const STREAM_RETRY_INTERVEL_MS: u64 = 6000;
    pub const STREAM_CONNECT_TIMEOUT_MS: u64 = 5000;
    pub const STREAM_NO_DATA_TIMEOUT_MS: u64 = 10000;
}

pub mod url;
pub mod flv;
