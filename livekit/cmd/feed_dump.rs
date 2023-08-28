use std::{path::PathBuf, io::Write, fs::{self, OpenOptions}};
use foundations::byterepr::ByteRepr;
use livekit_feed::package::{Package, JsonPackage};
use livekit_feed_stor_raw::{kvdump::{self, Row, KV}, crc32, Key};

/// export feed raw storage to jsonl file
#[derive(argh::FromArgs)]
#[argh(subcommand, name = "feed-dump")]
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
    /// start time (timestamp)
    #[argh(option, short = 'f')]
    from: Option<u64>,
    /// end time (timestamp)
    #[argh(option, short = 't')]
    to: Option<u64>,
}

#[derive(serde::Serialize)]
struct Record {
    roomid: u32,
    time: u64,
    #[serde(flatten)]
    inner: JsonPackage,
}

pub fn main(Args { raw_stor_path, export_path, roomid_list, from, to }: Args) {
    let roomid_list: Option<Vec<u32>> = roomid_list.map(|l| l.split(',').map(|roomid| roomid.parse::<u32>().expect("FATAL: invaild roomid")).collect());
    let mut export_file = OpenOptions::new().write(true).create(true).append(true).open(export_path).unwrap();

    for entry in fs::read_dir(raw_stor_path).unwrap() {
        let entry = entry.unwrap();
        if entry.file_type().unwrap().is_file() {
            let kv_file = OpenOptions::new().read(true).open(entry.path()).unwrap();
            let reader = kvdump::Reader::init(kv_file).unwrap();
            for row in reader {
                match row.unwrap() {
                    Row::Hash(_) | Row::End => { },
                    Row::KV(KV { scope, key, value }) => {
                        let roomid = u32::from_be_bytes(scope.as_ref().try_into().unwrap());
                        let Key { time, hash } = Key::from_bytes(key.as_ref().try_into().unwrap());
                        assert_eq!(hash, crc32(&value));
                        if if let Some(roomid_list) = &roomid_list { roomid_list.contains(&roomid) } else { true } {
                            if if let Some(from) = &from { time > *from } else { true } {
                                if if let Some(to) = &to { time < *to } else { true } {
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
    }
}
