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
}

#[tokio::main]
async fn main() {
    let args = Args::from_args();
    let connection = Connect::new(args.roomid).await.unwrap();
    let init = Message::Binary(Package::create_init_request(&connection).encode());
    let heartbeat = Message::Binary(Package::HeartbeatRequest().encode());

    let storage = DB::open_default(args.storage_path).unwrap();
    let (socket, _) = connect_async(connection.url.as_str()).await.unwrap();
    let (mut write, read) = socket.split();

    write.send(init).await.unwrap();

    spawn(async move {
        let mut interval = time::interval(Duration::from_secs(30));
        loop {
            interval.tick().await;
            write.send(heartbeat.clone()).await.unwrap();
        }
    });

    spawn(async move {
        read.for_each(|message| async {
            match message.unwrap() {
                Message::Binary(payload) => {
                    let package = Package::decode(&payload);
                    println!("{:?}", package);
                    storage.put(Timestamp::now().to_bytes(), payload).unwrap();
                },
                _ => panic!("unexpected received websocket message type"),
            }
        }).await
    });

    signal::ctrl_c().await.unwrap();

    println!("quit");

    // socket.close(None);
}
