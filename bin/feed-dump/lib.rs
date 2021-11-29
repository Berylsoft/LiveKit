use std::{convert::TryInto, fs::{File, OpenOptions}};
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
    payloads: JsonValue,
}

pub fn open(path: String) -> File {
    OpenOptions::new().write(true).append(true).open(path).unwrap()
}

pub fn record(k: &[u8], v: &[u8]) -> String {
    let packages = Package::decode(v).flatten();
    let payloads = if packages.len() == 1 {
        packages.into_iter().next().unwrap().to_json().unwrap()
    } else {
        let payloads: Vec<JsonValue> = packages.into_iter().map(|package| package.to_json().unwrap()).collect();
        serde_json::to_value(payloads).unwrap()
    };
    serde_json::to_string(&Record {
        time: i64::from_be_bytes(k.try_into().unwrap()),
        payloads,
    }).unwrap()
}
