use structopt::StructOpt;
use std::{io::Write, fs::OpenOptions};
use livekit_feed::{payload::Payload, package::Package};
use livekit_feed_storage::{open_db, open_storage};

#[derive(StructOpt)]
struct Args {
    #[structopt(short = "r", long)]
    roomid: u32,
    #[structopt(short = "s", long)]
    storage_path: String,
    #[structopt(short = "o", long)]
    export_path: String,
    #[structopt(long)]
    rocks_ver: Option<String>,
}

#[derive(serde::Serialize)]
struct Record {
    time: u64,
    payloads: serde_json::Value,
}

fn record(payload: Payload) -> String {
    serde_json::to_string(&Record {
        time: payload.time,
        payloads: Package::decode(payload.payload).unwrap().into_json().unwrap(),
    }).unwrap()
}

fn main() {
    let Args { roomid, storage_path, export_path, rocks_ver } = Args::from_args();

    let mut file = OpenOptions::new().write(true).create(true).append(true).open(export_path).unwrap();

    if let Some(rocks_ver) = rocks_ver {
        let storage = rocksdb::DB::open_default(format!("{}/{}-{}", storage_path, roomid, rocks_ver)).unwrap();
        for (k, v) in storage.iterator(rocksdb::IteratorMode::Start) {
            writeln!(file, "{}", record(Payload::from_kv(k, v))).unwrap();
        }
    } else {
        let db = open_db(storage_path).unwrap();
        let storage = open_storage(&db, roomid).unwrap();
        for kv in storage.iter() {
            let (k, v) = kv.unwrap();
            writeln!(file, "{}", record(Payload::from_kv(k, v))).unwrap();
        }
    }
}
