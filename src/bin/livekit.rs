use structopt::StructOpt;
use tokio::{spawn, signal, fs::read_to_string};
use livekit::{config::Config, room::Room};

#[derive(StructOpt)]
struct Args {
    #[structopt(short = "c", long)]
    config_path: String,
}

#[tokio::main]
async fn main() {
    let config = read_to_string(Args::from_args().config_path).await.unwrap();
    let config: Config = serde_json::from_str(config.as_str()).unwrap();

    let room = Room::init(&config.rooms[0], &config.general).await;

    spawn(room.client_thread());

    spawn(async move {
        let mut receiver = room.receive();
        loop {
            println!("{:?}", receiver.recv().await.unwrap());
        }
    });

    signal::ctrl_c().await.unwrap();
}
