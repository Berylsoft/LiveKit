use tokio::{spawn, signal};
use tokio::time::{self, Duration};
use futures_util::{StreamExt, SinkExt};
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};
use structopt::StructOpt;
use rocksdb::DB;
use livekit::{package::Package, connect::Connect, util::Timestamp};

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

    let connection = Connect::new(roomid).await.unwrap();
    let storage = DB::open_default(storage_path).unwrap();
    let (socket, _) = connect_async(connection.url.as_str()).await.unwrap();
    let (mut write, read) = socket.split();

    let init = Message::Binary(Package::create_init_request(&connection).encode());
    write.send(init).await.unwrap();

    spawn(async move {
        let heartbeat = Message::Binary(Package::HeartbeatRequest().encode());
        let mut interval = time::interval(Duration::from_secs(heartbeat_rate));
        loop {
            interval.tick().await;
            write.send(heartbeat.clone()).await.unwrap();
        }
    });

    spawn(async move {
        read.for_each(|maybe_message| async {
            match maybe_message {
                Ok(message) => match message {
                    Message::Binary(payload) => {
                        let package = Package::decode(&payload);
                        println!("{:?}", package);
                        storage.put(Timestamp::now().to_bytes(), payload).unwrap();
                    },
                    any_other => {
                        println!("{:?}", any_other);
                        panic!("unexpected received websocket message type");
                    },
                },
                Err(error) => panic!("{}", error),
            }
        }).await
    });

    signal::ctrl_c().await.unwrap();

    println!("quit");

    // socket.close(None);
}
