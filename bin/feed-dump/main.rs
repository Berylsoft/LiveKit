use structopt::StructOpt;
use livekit_feed_client::storage::open_storage;
use livekit_feed_dump::*;

fn main() {
    let Args { roomid, storage_path, export_path } = Args::from_args();
    
    let db = open_storage(storage_path).unwrap();
    let storage = db.open_tree(roomid.to_string()).unwrap();

    let mut file = open(export_path);

    for kv in storage.iter() {
        let (k, v) = kv.unwrap();
        record(k.as_ref(), v.as_ref(), &mut file);
    }
}
