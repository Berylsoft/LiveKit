use structopt::StructOpt;
use std::{io::Write, fs::OpenOptions};
use livekit_feed_client::storage::open_storage;
use livekit_feed_dump::*;

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

fn main() {
    let Args { roomid, storage_path, export_path, rocks_ver } = Args::from_args();

    let mut file = OpenOptions::new().write(true).create(true).append(true).open(export_path).unwrap();

    if let Some(rocks_ver) = rocks_ver {
        let storage = rocksdb::DB::open_default(format!("{}/{}-{}", storage_path, roomid, rocks_ver)).unwrap();
        for (k, v) in storage.iterator(rocksdb::IteratorMode::Start) {
            writeln!(file, "{}", record(k, v)).unwrap();
        }
    } else {
        let db = open_storage(storage_path).unwrap();
        let storage = db.open_tree(roomid.to_string()).unwrap();
        for kv in storage.iter() {
            let (k, v) = kv.unwrap();
            writeln!(file, "{}", record(k, v)).unwrap();
        }
    }
}
