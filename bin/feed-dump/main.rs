use std::{convert::TryInto, io::Write, fs::OpenOptions};
use serde_json::Value as JsonValue;
use structopt::StructOpt;
use livekit_feed_client::{storage::open_storage, package::Package};

#[derive(serde::Serialize)]
struct Record {
    time: i64,
    payloads: Vec<JsonValue>,
}

#[derive(StructOpt)]
struct Args {
    #[structopt(short = "r", long)]
    roomid: u32,
    #[structopt(short = "s", long)]
    storage_path: String,
    #[structopt(short = "o", long)]
    export_path: String,
}

fn main() {
    let Args { roomid, storage_path, export_path } = Args::from_args();
    
    let db = open_storage(storage_path).unwrap();
    let storage = db.open_tree(roomid.to_string()).unwrap();

    let mut file = OpenOptions::new().write(true).append(true).open(export_path).unwrap();

    for kv in storage.iter() {
        let (k, v) = kv.unwrap();
        let record = Record {
            time: i64::from_be_bytes(k.as_ref().try_into().unwrap()),
            payloads: Package::decode(v.as_ref()).flatten().into_iter().map(|package| package.to_json().unwrap()).collect(),
        };
        writeln!(file, "{}", serde_json::to_string(&record).unwrap()).unwrap();
    }
}
