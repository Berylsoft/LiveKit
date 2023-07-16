use std::path::PathBuf;
pub use crc32fast::hash as crc32;
pub use kvdump;
use kvdump::{KV, Config, Sizes, Result, actor::*};
use livekit_feed::stream::{Payload, now};

type Handle = actor::Handle<WriterContext>;

pub const IDENT: &str = "livekit-feed-raw";
pub const SIZES: Sizes = Sizes { scope: Some(4), key: Some(12), value: None };
pub const FILE_SYNC_INTERVAL_COUNT: u16 = 500;

#[derive(Debug)]
pub struct Key {
    pub time: u64,
    pub hash: u32,
}

impl Key {
    pub fn encode(&self) -> Box<[u8]> {
        [
            self.time.to_be_bytes().as_slice(),
            self.hash.to_be_bytes().as_slice(),
        ].concat().into_boxed_slice()
    }

    pub fn decode(raw: &[u8]) -> Key {
        assert_eq!(raw.len(), 12);
        let (time, hash) = raw.split_at(8);
        Key {
            time: u64::from_be_bytes(time.try_into().unwrap()),
            hash: u32::from_be_bytes(hash.try_into().unwrap()),
        }
    }

    pub fn from_payload(payload: &Payload) -> Key {
        Key {
            time: payload.time,
            hash: crc32(&payload.payload),
        }
    }
}

pub struct Writer {
    tx: Handle,
}

pub struct RoomWriter {
    roomid: u32,
    tx: Handle,
}

pub struct CloseHandle {
    tx: Handle,
}

impl Writer {
    pub async fn open(path: PathBuf) -> Result<(Writer, CloseHandle)> {
        let tx = actor::spawn(WriterContextConfig {
            path: path.join(now().to_string()),
            config: Config { ident: Box::from(IDENT.as_bytes()), sizes: SIZES.clone() },
            sync_interval: FILE_SYNC_INTERVAL_COUNT,
        }).await?;
        Ok((Writer { tx: tx.clone() }, CloseHandle { tx }))
    }

    pub fn open_room(&self, roomid: u32) -> RoomWriter {
        RoomWriter { roomid, tx: self.tx.clone() }
    }

    pub async fn write_hash(&self) -> Result<()> {
        self.tx.request(Request::Hash).await
    }

    pub async fn sync(&self) -> Result<()> {
        self.tx.request(Request::Sync).await
    }
}

impl RoomWriter {
    pub fn roomid(&self) -> u32 {
        self.roomid
    }

    pub async fn insert_payload(&self, payload: &Payload) -> std::result::Result<(), String> {
        let key = Key::from_payload(payload);
        self.tx.request(Request::KV(KV {
            scope: Box::from(self.roomid.to_be_bytes()),
            key: key.encode(),
            value: payload.payload.clone(),
        })).await.map_err(|err| format!(
            "[{: >10}] (stor-raw) FATAL: insert error: {:?} key={:?} val(hex)={}",
            self.roomid, err, key, hex::encode(&payload.payload),
        ))
    }
}

impl CloseHandle {
    #[inline(always)]
    pub async fn wait_close(self) -> Result<()> {
        self.tx.wait_close().await
    }
}
