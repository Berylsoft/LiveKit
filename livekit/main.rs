use structopt::StructOpt;
use tokio::{spawn, signal, fs::read_to_string};
use livekit_api::client::HttpClient;
use livekit_feed_storage::open_db;
use livekit::{config::*, room::Room};

#[derive(StructOpt)]
struct Args {
    #[structopt(short = "c", long)]
    config_path: String,
}

#[tokio::main]
async fn main() {
    env_logger::init();

    let config = read_to_string(Args::from_args().config_path).await.unwrap();
    let Groups { group } = toml::from_str(config.as_str()).unwrap();

    let http_client2 = HttpClient::new_bare().await;
    for Group { config, rooms } in group {
        let db = open_db(&config.storage.path).unwrap();
        let http_client = HttpConfig::build(config.http.clone()).await;
        for _sroomid in rooms {
            if _sroomid >= 0 {
                let sroomid = _sroomid.try_into().unwrap();
                let room = Room::init(sroomid, &config, &db, http_client.clone(), http_client2.clone()).await;
                if let Some(_) = &config.dump {
                    spawn(room.dump().await);
                } else {
                    panic!();
                }
                if let Some(_) = &config.record {
                    spawn(room.record());
                }
            }
        }
    }

    signal::ctrl_c().await.unwrap();
}
