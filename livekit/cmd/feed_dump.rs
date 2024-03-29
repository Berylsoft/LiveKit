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
    /// filter out HeartbeatResponse(1)
    #[argh(switch)]
    filter_out_heartbeat_eq1: bool,
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

pub fn main(Args { raw_stor_path, export_path, roomid_list, file, from, to, filter_list, filter_out_heartbeat_eq1 }: Args) {
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
        'iter_row: for row in reader {
            macro_rules! gate {
                (@opt $arg:ident: $pass_cond:block) => {
                    if let Some($arg) = &$arg {
                        if !$pass_cond {
                            continue 'iter_row;
                        }
                    }
                };
                (@bool $arg:ident: $pass_cond:block) => {
                    // equals if $arg then if !$pass_cond
                    if $arg != $pass_cond {
                        continue 'iter_row;
                    }
                };
            }

            match row.unwrap() {
                Row::Hash(_) | Row::End => { },
                Row::KV(KV { scope, key, value }) => {
                    let roomid = u32::from_be_bytes(scope.as_ref().try_into().unwrap());
                    let Key { time, hash } = Key::from_bytes(key.as_ref().try_into().unwrap());
                    assert_eq!(hash, crc32(&value));
                    gate!(@opt roomid_list: { roomid_list.contains(&roomid) });
                    gate!(@opt from: { time > *from });
                    gate!(@opt to: { time < *to });
                    let inner = Package::decode(&value).unwrap().to_json().unwrap();
                    gate!(@bool filter_out_heartbeat_eq1: { !matches!(inner, JsonPackage::HeartbeatResponse(1)) });
                    gate!(@opt filter_list: { get_single_cmd(&inner).map_or(true, |cmd| filter_list.contains(&cmd)) });
                    let record = Record { roomid, time, inner };
                    serde_json::to_writer(&mut export_file, &record).unwrap();
                    writeln!(export_file).unwrap();
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
