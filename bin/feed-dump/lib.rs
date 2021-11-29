use std::{convert::TryInto, io::Write, fs::{File, OpenOptions}};
use serde_json::Value as JsonValue;
use structopt::StructOpt;
use livekit_feed_client::package::Package;

#[derive(StructOpt)]
pub struct Args {
    #[structopt(short = "r", long)]
    pub roomid: u32,
    #[structopt(short = "s", long)]
    pub storage_path: String,
    #[structopt(short = "o", long)]
    pub export_path: String,
}

#[derive(serde::Serialize)]
pub struct Record {
    time: i64,
    payloads: Vec<JsonValue>,
}

pub fn open(export_path: String) -> File {
    OpenOptions::new().write(true).append(true).open(export_path).unwrap()
}

pub fn record(k: &[u8], v: &[u8], file: &mut File) {
    let record = Record {
        time: i64::from_be_bytes(k.try_into().unwrap()),
        payloads: Package::decode(v).flatten().into_iter().map(|package| package.to_json().unwrap()).collect(),
    };
    writeln!(file, "{}", serde_json::to_string(&record).unwrap()).unwrap();
}
