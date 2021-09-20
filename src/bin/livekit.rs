use structopt::StructOpt;
use tokio::{spawn, signal, fs::read_to_string};
use livekit::{
    config::Config,
    client::client_thread,
    room::Room,
};

#[derive(StructOpt)]
struct Args {
    #[structopt(short = "c", long)]
    config_path: String,
}

#[tokio::main]
async fn main() {
    let config = read_to_string(Args::from_args().config_path).await.unwrap();
    let config: Config = serde_json::from_str(config.as_str()).unwrap();

    let (room, storage) = Room::init(&config.rooms[0], &config.general).await;

    spawn(client_thread(room.id(), room.sender(), storage));

    let mut receiver = room.receiver();

    spawn(async move {
        loop {
            println!("{:?}", receiver.recv().await.unwrap());
        }
    });

    signal::ctrl_c().await.unwrap();
}
