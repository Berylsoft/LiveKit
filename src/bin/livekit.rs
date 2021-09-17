use tokio::spawn;
use tokio::time::{self, Duration};
use futures_util::{StreamExt, SinkExt};
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};
use structopt::StructOpt;
use livekit::{package::Package, connect::Connect};

#[derive(StructOpt)]
struct Args {
    #[structopt(short = "r", long)]
    roomid: u32,
}

#[tokio::main]
async fn main() {
    let Args { roomid } = Args::from_args();
    let connection = Connect::new(roomid).await.unwrap();
    let init = Message::Binary(Package::create_init_request(&connection).encode());
    let heartbeat = Message::Binary(Package::HeartbeatRequest().encode());

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

    read.for_each(|message| async {
        match message.unwrap() {
            Message::Binary(payload) => println!("{:?}", Package::decode(&payload)),
            _ => panic!("unexpected websocket message type"),
        }
    }).await;

    // socket.close(None);
}
