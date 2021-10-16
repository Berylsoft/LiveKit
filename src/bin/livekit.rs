use structopt::StructOpt;
use tokio::{spawn, signal, fs::read_to_string};
use livekit::{
    config::General as GeneralConfig,
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
            spawn(room.print_events_to_stdout());
        }
    }

    signal::ctrl_c().await.unwrap();
}
