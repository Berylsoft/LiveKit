use tokio::{spawn, signal, sync::broadcast::channel};
use structopt::StructOpt;
use rocksdb::DB;
use livekit::client::{repeater, Event};

#[derive(StructOpt)]
struct Args {
    #[structopt(short = "r", long)]
    roomid: u32,
    #[structopt(short = "p", long)]
    storage_path: String,
    #[structopt(long, default_value = "30")]
    heartbeat_rate: u64,
}

#[tokio::main]
async fn main() {
    let Args { roomid, storage_path, heartbeat_rate } = Args::from_args();
    assert!(heartbeat_rate < 60 && heartbeat_rate >= 1);

    let storage = DB::open_default(storage_path).unwrap();
    let (channel_tx, mut channel_rx) = channel(32);

    spawn(async move {
        for _ in 1..2 {
            channel_tx.send(Event::Open).unwrap();
            if let Err(error) = repeater(roomid, &mut channel_tx.clone(), &storage).await {
                channel_tx.send(Event::Close).unwrap();
                println!("!> {}", error);
            };
        }
    });

    spawn(async move {
        loop {
            println!("{:?}", channel_rx.recv().await.unwrap());
        }
    });

    signal::ctrl_c().await.unwrap();
}
