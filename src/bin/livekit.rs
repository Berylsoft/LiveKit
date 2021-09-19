use structopt::StructOpt;
use tokio::{spawn, signal, sync::broadcast::channel, fs::read_to_string};
use rocksdb::DB;
use livekit::{config::{Config, EVENT_CHANNEL_BUFFER_SIZE}, client::{repeater, Event}};

#[derive(StructOpt)]
struct Args {
    #[structopt(short = "c", long)]
    config_path: String,
}

#[tokio::main]
async fn main() {
    let config = read_to_string(Args::from_args().config_path).await.unwrap();
    let config: Config = serde_json::from_str(config.as_str()).unwrap();

    let room = &config.rooms[0];
    let roomid = room.roomid;
    let storage_path = room.storage_name();

    let storage = DB::open_default(format!("{}/{}", config.storage_root, storage_path)).unwrap();
    let (channel_tx, mut channel_rx) = channel(EVENT_CHANNEL_BUFFER_SIZE);

    spawn(async move {
        for _ in 1..2 {
            channel_tx.send(Event::Open).unwrap();
            if let Err(error) = repeater(roomid, &mut channel_tx.clone(), &storage).await {
                channel_tx.send(Event::Close).unwrap();
                eprintln!("!> {}", error);
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
