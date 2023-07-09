use std::path::Path;
use tokio::fs::{self, OpenOptions, File};
pub use crc32fast::hash as crc32;
mod actor;
use actor::{ReqTx, CloseHandle};
pub use kvdump;
mod async_kvdump;
use async_kvdump::AsyncWriter;
use kvdump::{KV, Config, Sizes, Error, Result};
use livekit_feed::stream::{Payload, now};

pub const IDENT: &str = "livekit-feed-raw";
pub const SIZES: Sizes = Sizes { scope: Some(4), key: Some(12), value: None };

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

#[derive(Debug, Clone)]
enum Request {
    KV(KV),
    Hash,
    Sync,
}

struct WriterContext {
    writer: AsyncWriter<File>,
}

impl WriterContext {
    async fn exec(&mut self, req: Request) -> Result<()> {
        match req {
            Request::KV(kv) => self.writer.write_kv(kv).await,
            Request::Hash => self.writer.write_hash().await.map(|_| ()),
            Request::Sync => self.writer.datasync().await,
        }
    }

    async fn close(mut self) {
        self.writer.close_file().await.expect("FATAL: Error occurred during closing");
    }
}

pub struct Writer {
    tx: ReqTx,
}

pub struct RoomWriter {
    roomid: u32,
    tx: ReqTx,
}

impl Writer {
    pub async fn open<P: AsRef<Path>>(path: P) -> Result<(Writer, CloseHandle)> {
        let writer = {
            fs::create_dir_all(&path).await?;
            let path = path.as_ref().join(now().to_string());
            let file = OpenOptions::new().write(true).create_new(true).open(path).await?;
            let config = Config { ident: Box::from(IDENT.as_bytes()), sizes: SIZES.clone() };
            AsyncWriter::init(file, config).await?
        };
        let (tx, close) = actor::spawn(WriterContext { writer });
        Ok((Writer { tx }, close))
    }

    pub fn open_room(&self, roomid: u32) -> RoomWriter {
        RoomWriter { roomid, tx: self.tx.clone() }
    }

    pub async fn write_hash(&self) -> Result<()> {
        self.tx.request(Request::Hash).await.unwrap_or(Err(Error::AsyncFileClosed))
    }

    pub async fn sync(&self) -> Result<()> {
        self.tx.request(Request::Sync).await.unwrap_or(Err(Error::AsyncFileClosed))
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
        })).await.unwrap_or(Err(Error::AsyncFileClosed)).map_err(|err| format!(
            "[{: >10}] (stor-raw) FATAL: insert error: {:?} key={:?} val(hex)={}",
            self.roomid, err, key, hex::encode(&payload.payload),
        ))
    }
}
