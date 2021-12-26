pub use sled;

use self::sled::*;
use livekit_feed::{config::*, stream::FeedStreamPayload};

pub fn open_db<P: AsRef<std::path::Path>>(path: P) -> Result<Db> {
    Config::default()
        .path(path)
        .cache_capacity(FEED_STORAGE_CACHE_MAX_BYTE)
        .flush_every_ms(Some(FEED_STORAGE_FLUSH_INTERVAL_MS))
        .open()
}

pub fn open_storage(db: &Db, roomid: u32) -> Result<Tree> {
    db.open_tree(roomid.to_string())
}

pub fn insert_payload(storage: &Tree, payload: &FeedStreamPayload) -> Result<Option<IVec>> {
    storage.insert(payload.time.to_bytes(), payload.payload.as_slice())
}

#[cfg(feature = "rec")]
mod rec;
#[cfg(feature = "rec")]
pub use rec::rec;
