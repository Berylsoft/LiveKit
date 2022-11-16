use std::{path::Path, fs::{self, OpenOptions, File}};
pub use crc32fast::hash as crc32;
use actor::{Executor, ReqTx, CloseHandle};
use kvdump::{KV, Config, Sizes, Error, Result};
pub use kvdump;
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
}

struct WriterContext {
    writer: kvdump::Writer<File>,
}

impl Executor for WriterContext {
    type Req = Request;
    type Res = Result<()>;

    fn exec(&mut self, req: Self::Req) -> Self::Res {
        match req {
            Request::KV(kv) => self.writer.write_kv(kv),
            Request::Hash => self.writer.write_hash().map(|_| ()),
        }
    }
}

pub struct Writer {
    tx: ReqTx<WriterContext>,
}

pub struct RoomWriter {
    roomid: u32,
    tx: ReqTx<WriterContext>,
}

impl Writer {
    pub fn open<P: AsRef<Path>>(path: P) -> Result<(Writer, CloseHandle)> {
        let writer = {
            fs::create_dir_all(&path)?;
            let path = path.as_ref().join(now().to_string());
            let file = OpenOptions::new().write(true).create_new(true).open(path)?;
            let config = Config { ident: Box::from(IDENT.as_bytes()), sizes: SIZES.clone() };
            kvdump::Writer::init(file, config)?
        };
        let (tx, close) = actor::spawn::<WriterContext>(WriterContext { writer });
        Ok((Writer { tx }, close))
    }

    pub fn open_room(&self, roomid: u32) -> RoomWriter {
        RoomWriter { roomid, tx: self.tx.clone() }
    }

    pub async fn write_hash(&self) -> Result<()> {
        self.tx.request(Request::Hash).await.unwrap_or(Err(Error::AsyncFileClosed))
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
