use structopt::StructOpt;
use tokio::{spawn, signal, fs::read_to_string};
use livekit_api::client::HttpClient;
use livekit::{
    storage::open_storage,
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

    let http_client2 = HttpClient::new_bare().await;
    for Group { config, rooms } in groups {
        let db = open_storage(&config.common.storage_path).unwrap();
        let http_client = HttpClient::new(config.common.access_token.clone(), config.common.api_proxy.clone()).await;
        for room in rooms {
            if room.operational {
                let room = Room::init(room.sroomid, config.clone(), &db, http_client.clone(), http_client2.clone()).await;
                spawn(room.print_events_to_stdout());
                if let Some(record) = room.record() {
                    spawn(record);
                }
            }
        }
    }

    signal::ctrl_c().await.unwrap();
}