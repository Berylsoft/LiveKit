use structopt::StructOpt;
use tokio::{spawn, signal, fs::read_to_string};
use livekit::{
    config::Group,
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
    let groups: Vec<Group> = serde_json::from_str(config.as_str()).unwrap();

    for Group { config, rooms } in groups {
        for room in rooms {
            if room.operational {
                let room = Room::init(room.sroomid, config.clone()).await;
                spawn(room.print_events_to_stdout());
                if let Some(record) = room.record() {
                    spawn(record);
                }
            }
        }
    }

    signal::ctrl_c().await.unwrap();
}
