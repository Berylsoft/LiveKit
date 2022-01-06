use structopt::StructOpt;
use std::{path::PathBuf, io::Write, fs::{File, OpenOptions}};
use livekit_feed::{payload::Payload, package::Package};
use livekit_feed_storage::{open_db, open_storage};

#[derive(StructOpt)]
struct Args {
    #[structopt(short = "r", long)]
    roomid: u32,
    #[structopt(short = "s", long, parse(from_os_str))]
    storage_path: PathBuf,
    #[structopt(short = "o", long, parse(from_os_str))]
    export_path: PathBuf,
    // #[cfg(feature = "rocks")]
    #[structopt(long)]
    rocks_ver: Option<String>,
}

#[derive(serde::Serialize)]
struct Record {
    time: u64,
    payloads: serde_json::Value,
}

fn record<Au8: AsRef<[u8]>>(file: &mut File, kv: (Au8, Au8)) {
    let Payload { time, payload } = Payload::from_kv(kv);
    serde_json::to_writer(&*file, &Record {
        time,
        payloads: Package::decode(payload).unwrap().into_json().unwrap(),
    }).unwrap();
    writeln!(file).unwrap();
}

fn main() {
    let Args { roomid, storage_path, export_path, rocks_ver } = Args::from_args();

    let mut file = OpenOptions::new().write(true).create(true).append(true).open(export_path).unwrap();

    if let Some(rocks_ver) = rocks_ver {
        #[cfg(feature = "rocks")]
        {
            let storage = rocksdb::DB::open_default(format!("{}/{}-{}", storage_path, roomid, rocks_ver)).unwrap();
            for kv in storage.iterator(rocksdb::IteratorMode::Start) {
                record(&mut file, kv);
            }
        }
        #[cfg(not(feature = "rocks"))]
        {
            let _ = rocks_ver;
            panic!("complied without rocksdb");
        }
    } else {
        let db = open_db(storage_path).unwrap();
        let storage = open_storage(&db, roomid).unwrap();
        for kv in storage.iter() {
            record(&mut file, kv.unwrap());
        }
    }
}
