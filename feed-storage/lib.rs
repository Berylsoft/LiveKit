pub use async_kvdump;
pub use self::async_kvdump::{Db, Scope};

use std::fs;
use self::async_kvdump::*;
use livekit_feed::{util::now, payload::Payload};

pub const CONFIG_IDENT: &str = "livekit-feed-raw";
pub const CONFIG_SIZES: Sizes = Sizes { scope: Some(4), key: Some(12), value: None };

pub fn open_db<P: AsRef<std::path::Path>>(path: P) -> Result<Db> {
    fs::create_dir_all(&path)?;
    let mut path = path.as_ref().to_owned();
    path.push(now().to_string());
    let config = Config {
        ident: Box::from(CONFIG_IDENT.as_bytes()),
        sizes: CONFIG_SIZES.clone()
    };
    Db::init(path, config)
}

pub fn open_storage(db: &Db, roomid: u32) -> Scope {
    db.open_scope(roomid.to_be_bytes())
}

fn roomid_of(storage: &Scope) -> u32 {
    u32::from_be_bytes(storage.name().as_ref().try_into().unwrap())
}

pub async fn insert_payload(storage: &Scope, payload: &Payload) {
    let key = payload.get_key();
    match storage.write_kv(key.encode(), payload.payload.as_slice()).await {
        Ok(()) => { },
        Err(err) => panic!(
            "[{: >10}] (storage) FATAL: insert error: {:?} key={:?} val(hex)={}",
            roomid_of(storage), err, key, hex::encode(&payload.payload),
        ),
    }
}

#[cfg(feature = "rec")]
mod rec;
#[cfg(feature = "rec")]
pub use rec::rec;
