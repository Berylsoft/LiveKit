use std::{convert::TryInto, io::Write, fs::OpenOptions};
use serde_json::{Value as JsonValue, json};
use structopt::StructOpt;
use livekit_feed_client::{storage::open_storage, package::{Package, FlatPackage}};

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
        let payloads: Vec<JsonValue> = Package::decode(v).flatten().into_iter().map(|package| {
            match package {
                FlatPackage::Json(payload) => serde_json::from_str(payload.as_str()).unwrap(),
                FlatPackage::HeartbeatResponse(num) => json!(num),
                FlatPackage::InitResponse(payload) => serde_json::from_str(payload.as_str()).unwrap(),
                FlatPackage::CodecError(_, error) => panic!("{:?}", error),
            }
        }).collect();
        let record = Record {
            time: i64::from_be_bytes(k.as_ref().try_into().unwrap()),
            payloads,
        };
        writeln!(file, "{}", serde_json::to_string(&record).unwrap()).unwrap();
    }
}
