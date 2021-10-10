use structopt::StructOpt;
use tokio::{spawn, signal, fs::read_to_string, sync::broadcast::channel};
use livekit::{
    config::{General as GeneralConfig, EVENT_CHANNEL_BUFFER_SIZE},
    feed::client::{client, open_storage},
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
            let (sender, mut receiver) = channel(EVENT_CHANNEL_BUFFER_SIZE);
            let storage = open_storage(room.storage_path());
            spawn(client(room.id(), sender, storage));

            spawn(async move {
                loop {
                    println!("{:?}", receiver.recv().await.unwrap());
                }
            });
        }
    }

    signal::ctrl_c().await.unwrap();
}
