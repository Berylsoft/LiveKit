use structopt::StructOpt;
use tokio::{spawn, signal, fs::read_to_string};
use livekit::{
    config::General as GeneralConfig,
    feed::client::client,
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
            let (room, storage) = Room::init(&room, &group.config).await;
            spawn(client(room.id(), room.sender(), storage));

            let mut receiver = room.receiver();
            spawn(async move {
                loop {
                    println!("{:?}", receiver.recv().await.unwrap());
                }
            });
        }
    }

    signal::ctrl_c().await.unwrap();
}
