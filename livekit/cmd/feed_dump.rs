use std::{path::PathBuf, io::{Write, stdout}, fs::{self, OpenOptions}};
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
    /// export jsonl file path and name (default stdout)
    #[argh(option, short = 'o')]
    export_path: Option<PathBuf>,
    /// comma-separated list of roomid (no short id) to export (default all)
    #[argh(option, short = 'r')]
    roomid_list: Option<String>,
    /// read file rather than dir
    #[argh(switch)]
    file: bool,
    /// start time (timestamp)
    #[argh(option)]
    from: Option<u64>,
    /// end time (timestamp)
    #[argh(option)]
    to: Option<u64>,
    /// comma-separated list of not included single cmd (not for multi)
    #[argh(option, long = "filter-out")]
    filter_list: Option<String>,
}

#[derive(serde::Serialize)]
struct Record {
    roomid: u32,
    time: u64,
    #[serde(flatten)]
    inner: JsonPackage,
}

fn get_single_cmd(pkg: &JsonPackage) -> Option<&str> {
    if let JsonPackage::Json(json) = pkg {
        Some(json.as_object()?.get("cmd")?.as_str()?)
    } else {
        None
    }
}

pub fn main(Args { raw_stor_path, export_path, roomid_list, file, from, to, filter_list }: Args) {
    let roomid_list: Option<Vec<u32>> = roomid_list.map(|l| l.split(',').map(|roomid| roomid.parse::<u32>().expect("FATAL: invaild roomid")).collect());
    let filter_list: Option<&str> = filter_list.as_deref();
    let filter_list: Option<Vec<&str>> = filter_list.map(|l| l.split(',').collect());
    let mut export_file: Box<dyn Write> = if let Some(path) = export_path {
        Box::new(OpenOptions::new().write(true).create(true).append(true).open(path).unwrap())
    } else {
        Box::new(stdout().lock())
    };

    let mut handle_file = move |path: PathBuf| {
        let kv_file = OpenOptions::new().read(true).open(path).unwrap();
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
                                if if let Some(filter_list) = filter_list.as_deref() {
                                    if let Some(cmd) = get_single_cmd(&inner) {
                                        let mut ret = true;
                                        for filtered_cmd in filter_list {
                                            if *filtered_cmd == cmd {
                                                ret = false;
                                                break;
                                            }
                                        }
                                        ret
                                    } else { true }
                                } else { true } {
                                    let record = Record { roomid, time, inner };
                                    serde_json::to_writer(&mut export_file, &record).unwrap();
                                    writeln!(export_file).unwrap();
                                }
                            }
                        }
                    }
                }
            }
        }
    };

    if file {
        handle_file(raw_stor_path);
    } else {
        for entry in fs::read_dir(raw_stor_path).unwrap() {
            let entry = entry.unwrap();
            if entry.file_type().unwrap().is_file() {
                handle_file(entry.path());
            }
        }
    }
}
