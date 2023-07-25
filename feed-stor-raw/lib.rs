use std::path::PathBuf;
use bytes::Bytes;
pub use crc32fast::hash as crc32;
use foundations::{byterepr_struct, byterepr::ByteRepr};
pub use kvdump;
use kvdump::{KV, Sizes, Result, actor::{Request, WriterContextConfig}};
use livekit_feed::stream::{Payload, now};

type WriterContext = kvdump::actor::WriterContext<Config, FILE_SYNC_INTERVAL_COUNT>;
type Handle = tokio_actor::Handle<WriterContext>;

pub const IDENT: &str = "livekit-feed-raw";
pub const SIZES: Sizes = Sizes { scope: Some(4), key: Some(12), value: None };
pub const FILE_SYNC_INTERVAL_COUNT: u16 = 500;

pub struct Config;

impl kvdump::Config for Config {
    fn ident<'a>(&'a self) -> &'a [u8] {
        IDENT.as_bytes()
    }

    fn sizes<'a>(&'a self) -> &'a Sizes {
        &SIZES
    }
}

byterepr_struct! {
    #[derive(Debug)]
    pub struct Key {
        pub time: u64,
        pub hash: u32,
    }
}

impl Key {
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
    roomid_bytes: Bytes,
    tx: Handle,
}

pub struct CloseHandle {
    tx: Handle,
}

impl Writer {
    pub async fn open(path: PathBuf) -> Result<(Writer, CloseHandle)> {
        tokio::fs::create_dir_all(&path).await?;
        let tx = tokio_actor::spawn_async(WriterContextConfig {
            path: path.join(now().to_string()),
            config: Config,
        }).await?;
        Ok((Writer { tx: tx.clone() }, CloseHandle { tx }))
    }

    pub fn open_room(&self, roomid: u32) -> RoomWriter {
        RoomWriter { roomid, roomid_bytes: Bytes::copy_from_slice(&roomid.to_be_bytes()), tx: self.tx.clone() }
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
            scope: self.roomid_bytes.clone(),
            key: Bytes::copy_from_slice(&key.to_bytes()),
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
