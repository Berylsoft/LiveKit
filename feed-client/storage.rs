pub use sled;

use self::sled::*;

pub fn open_db<P: AsRef<std::path::Path>>(path: P) -> Result<Db> {
    use livekit_feed::config::*;
    Config::default()
        .path(path)
        .cache_capacity(FEED_STORAGE_CACHE_MAX_BYTE)
        .flush_every_ms(Some(FEED_STORAGE_FLUSH_INTERVAL_MS))
        .open()
}

pub fn open_storage(db: &Db, roomid: u32) -> Result<Tree> {
    db.open_tree(roomid.to_string())
}

pub(crate) trait Insert {
    // fn as_slices(&self) -> (&[u8], &[u8]);

    fn insert(&self, storage: &Tree) -> Result<Option<IVec>>;
}

impl Insert for livekit_feed::stream::FeedStreamPayload {
    fn insert(&self, storage: &Tree) -> Result<Option<IVec>> {
        storage.insert(self.time.to_bytes(), self.payload.as_slice())
    }
}
