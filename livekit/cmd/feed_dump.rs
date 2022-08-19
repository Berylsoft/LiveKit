use std::{path::PathBuf, io::Write, fs::{self, OpenOptions}};
use livekit_feed::package::{Package, JsonPackage};
use livekit_feed_stor_raw::{kvdump::{self, Row, KV}, crc32, Key};

#[derive(argh::FromArgs)]
#[argh(subcommand, name = "feed-dump")]
/// export feed raw storage to jsonl file
pub struct Args {
    /// feed raw storage directory path
    #[argh(option, short = 'i')]
    raw_stor_path: PathBuf,
    /// export jsonl file path and name
    #[argh(option, short = 'o')]
    export_path: PathBuf,
    /// comma-separated list of roomid (no short id) to export (default all)
    #[argh(option, short = 'r')]
    roomid_list: Option<String>,
}

#[derive(serde::Serialize)]
struct Record {
    roomid: u32,
    time: u64,
    #[serde(flatten)]
    inner: JsonPackage,
}

pub fn main(Args { raw_stor_path, export_path, roomid_list }: Args) {
    let roomid_list: Option<Vec<u32>> = roomid_list.map(|l| l.split(",").map(|roomid| roomid.parse::<u32>().expect("FATAL: invaild roomid")).collect());
    let mut export_file = OpenOptions::new().write(true).create(true).append(true).open(export_path).unwrap();

    for entry in fs::read_dir(raw_stor_path).unwrap() {
        let entry = entry.unwrap();
        if entry.file_type().unwrap().is_file() {
            let kv_file = OpenOptions::new().read(true).open(entry.path()).unwrap();
            let mut reader = kvdump::Reader::init(kv_file).unwrap();
            while let Some(row) = reader.next() {
                match row.unwrap() {
                    Row::Hash(_) | Row::End => { },
                    Row::KV(KV { scope, key, value }) => {
                        let roomid = u32::from_be_bytes(scope.as_ref().try_into().unwrap());
                        if if let Some(roomid_list) = &roomid_list { roomid_list.contains(&roomid) } else { true } {
                            let Key { time, hash } = Key::decode(&key);
                            assert_eq!(hash, crc32(&value));
                            let inner = Package::decode(&value).unwrap().to_json().unwrap();
                            let record = Record { roomid, time, inner };
                            serde_json::to_writer(&export_file, &record).unwrap();
                            writeln!(export_file).unwrap();
                        }
                    }
                }
            }
        }
    }
}
