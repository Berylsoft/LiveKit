use structopt::StructOpt;
use tokio::{spawn, signal, fs::read_to_string};
use livekit::{
    config::General as GeneralConfig,
    feed,
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
    let general_config: GeneralConfig = serde_json::from_str(config.as_str()).unwrap();

    for group in general_config.groups {
        for room in group.rooms {
            let room = Room::init(&room, &group.config).await;
            let mut receiver = feed::client::init(room.id(), room.storage_path());

            spawn(async move {
                loop {
                    println!("{:?}", receiver.recv().await.unwrap());
                }
            });
        }
    }

    signal::ctrl_c().await.unwrap();
}
