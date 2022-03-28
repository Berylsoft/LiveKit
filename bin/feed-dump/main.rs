use structopt::StructOpt;
use std::{path::PathBuf, io::Write, fs::OpenOptions};
use kvdump::{Reader, KV, Row};
use livekit_feed::{payload::Payload, package::Package};

#[derive(StructOpt)]
struct Args {
    #[structopt(short = "r", long)]
    roomid: u32,
    #[structopt(short = "s", long, parse(from_os_str))]
    storage_path: PathBuf,
    #[structopt(short = "o", long, parse(from_os_str))]
    export_path: PathBuf,
}

#[derive(serde::Serialize)]
struct Record {
    time: u64,
    payloads: serde_json::Value,
}

fn main() {
    let Args { roomid, storage_path, export_path } = Args::from_args();

    let mut file = OpenOptions::new().write(true).create(true).append(true).open(export_path).unwrap();
    let db = OpenOptions::new().read(true).open(storage_path).unwrap();
    let mut reader = Reader::init(db).unwrap();

    while let Some(kv) = reader.next() {
        if let Row::KV(KV { scope, key, value }) = kv.unwrap() {
            if scope.as_ref() == roomid.to_be_bytes() {
                let Payload { time, payload } = Payload::from_kv(key, value);
                serde_json::to_writer(&file, &Record {
                    time,
                    payloads: Package::decode(payload).unwrap().into_json().unwrap(),
                }).unwrap();
                writeln!(file).unwrap();
            }
        }
    }
}
