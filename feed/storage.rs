pub use async_kvdump;
pub use self::async_kvdump::{Db, Scope, KV};

use self::async_kvdump::*;
use crate::{config::*, util::now, payload::Payload};

pub fn config() -> Config {
    Config {
        ident: Box::from(FEED_STORAGE_IDENT.as_bytes()),
        sizes: Sizes {
            scope: Some(FEED_STORAGE_SCOPE_LEN),
            key: Some(FEED_STORAGE_KEY_LEN),
            value: None,
        }
    }
}

pub fn open_db<P: AsRef<std::path::Path>>(path: P) -> Result<Db> {
    std::fs::create_dir_all(&path)?;
    Db::init(path.as_ref().join(now().to_string()), config())
}

pub fn open_storage(db: &Db, roomid: u32) -> Scope {
    db.open_scope(roomid.to_be_bytes())
}

fn roomid_of(storage: &Scope) -> u32 {
    u32::from_be_bytes(storage.name().as_ref().try_into().unwrap())
}

pub async fn insert_payload(storage: &Scope, payload: &Payload) {
    let key = payload.get_key();
    match storage.write_kv(key.encode(), payload.payload.as_ref()).await {
        Ok(()) => { },
        Err(err) => panic!(
            "[{: >10}] (storage) FATAL: insert error: {:?} key={:?} val(hex)={}",
            roomid_of(storage), err, key, hex::encode(&payload.payload),
        ),
    }
}
