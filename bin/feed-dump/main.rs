use structopt::StructOpt;
use std::{path::PathBuf, io::Write, fs::{self, OpenOptions}};
use kvdump::{Reader, KV, Row};
use livekit_feed::{payload::Payload, package::Package, storage::config};

#[derive(StructOpt)]
struct Args {
    #[structopt(short = "s", long, parse(from_os_str))]
    storage_path: PathBuf,
    #[structopt(short = "o", long, parse(from_os_str))]
    export_path: PathBuf,
    #[structopt(short = "r", long)]
    roomid: Option<u32>,
}

#[derive(serde::Serialize)]
struct Record {
    roomid: u32,
    time: u64,
    payloads: serde_json::Value,
}

fn main() {
    let Args { roomid, storage_path, export_path } = Args::from_args();
    let mut file = OpenOptions::new().write(true).create(true).append(true).open(export_path).unwrap();

    for entry in fs::read_dir(storage_path).unwrap() {
        let entry = entry.unwrap();
        if entry.file_type().unwrap().is_file() {
            let db_file = OpenOptions::new().read(true).open(entry.path()).unwrap();
            let mut reader = Reader::init(db_file).unwrap();
            assert_eq!(reader.config(), &config());
            while let Some(row) = reader.next() {
                match row.unwrap() {
                    Row::Hash(_) | Row::End => { },
                    Row::KV(KV { scope, key, value }) => {
                        let flag = if let Some(roomid) = roomid {
                            scope.as_ref() == roomid.to_be_bytes()
                        } else {
                            true
                        };
                        if flag {
                            let Payload { time, payload } = Payload::from_kv(key, value);
                            serde_json::to_writer(&file, &Record {
                                roomid: roomid.unwrap_or_else(|| u32::from_be_bytes(scope.as_ref().try_into().unwrap())),
                                time,
                                payloads: Package::decode(payload).unwrap().into_json().unwrap(),
                            }).unwrap();
                            writeln!(file).unwrap();
                        }
                    }
                }
            }
        }
    }
}
