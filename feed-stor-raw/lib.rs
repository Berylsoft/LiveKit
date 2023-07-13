use std::{path::{Path, PathBuf}, fs::{self, OpenOptions, File}};
pub use crc32fast::hash as crc32;
mod actor;
use actor::{ReqTx, CloseHandle};
pub use kvdump;
use kvdump::{KV, Config, Sizes, Error, Result};
use livekit_feed::stream::{Payload, now};

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

#[derive(Debug, Clone)]
enum Request {
    KV(KV),
    Hash,
    Sync,
    Close,
}

struct WriterContext {
    writer: kvdump::Writer<File>,
    non_synced_count: u16,
}

impl WriterContext {
    fn init(path: &Path) -> Result<WriterContext> {
        fs::create_dir_all(&path)?;
        let path = path.join(now().to_string());
        let file = OpenOptions::new().write(true).create_new(true).open(path)?;
        let config = Config { ident: Box::from(IDENT.as_bytes()), sizes: SIZES.clone() };
        Ok(WriterContext { writer: kvdump::Writer::init(file, config)?, non_synced_count: 0 })
    }

    fn exec(&mut self, req: Request) -> Result<()> {
        match req {
            Request::KV(kv) => {
                self.writer.write_kv(kv)?;
                self.non_synced_count += 1;
                if self.non_synced_count >= FILE_SYNC_INTERVAL_COUNT {
                    self.writer.datasync()?;
                    self.non_synced_count = 0;
                }
            },
            Request::Hash => {
                let _ = self.writer.write_hash()?;
            },
            Request::Sync => {
                self.writer.datasync()?;
            },
            Request::Close => {
                self.writer.close_file()?;
            }
        }
        Ok(())
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
    pub async fn open(path: PathBuf) -> Result<(Writer, CloseHandle)> {
        let (tx, close) = actor::spawn(path).await?;
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
