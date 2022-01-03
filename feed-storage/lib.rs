pub use sled;
pub use self::sled::{Db, Tree};

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

fn roomid(storage: &Tree) -> u32 {
    std::str::from_utf8(storage.name().as_ref()).unwrap().parse().unwrap()
}

pub fn insert_payload(storage: &Tree, payload: &FeedStreamPayload) {
    match storage.insert(payload.time.to_bytes(), payload.payload.as_slice()) {
        Ok(None) => { },
        Ok(Some(vec)) => log::error!(
            "[{: >10}] (storage) dup: key={} val(hex)={} val_prev(hex)={}",
            roomid(storage), payload.time.digits(), hex::encode(&payload.payload), hex::encode(vec),
        ),
        Err(err) => panic!(
            "[{: >10}] (storage) FATAL: insert error: {:?} key={} val(hex)={}",
            roomid(storage), err, payload.time.digits(), hex::encode(&payload.payload),
        ),
    }
}

#[cfg(feature = "rec")]
mod rec;
#[cfg(feature = "rec")]
pub use rec::rec;