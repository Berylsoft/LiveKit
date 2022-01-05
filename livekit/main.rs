use std::path::PathBuf;
use structopt::StructOpt;
use tokio::{spawn, signal, fs};
use livekit_api::client::HttpClient;
use livekit_feed_storage::open_db;
use livekit::{config::*, room::Room};

#[derive(StructOpt)]
struct Args {
    #[structopt(short = "c", long, parse(from_os_str))]
    config_path: PathBuf,
}

#[tokio::main]
async fn main() {
    env_logger::init();

    let config = fs::read_to_string(Args::from_args().config_path).await.expect("loading config error");
    let Groups { group } = toml::from_str(config.as_str()).expect("parsing config error");

    let http_client2 = HttpClient::new_bare().await;
    for Group { config, rooms } in group {
        if let Some(dump_config) = &config.dump {
            fs::create_dir_all(&dump_config.path).await.expect("creating dump directory error");
        }
        if let Some(record_config) = &config.record {
            fs::create_dir_all(&record_config.path).await.expect("creating record directory error");
        }
        let db = open_db(&config.storage.path).expect("opening storage error");
        let http_client = HttpConfig::build(config.http.clone()).await;
        for _sroomid in rooms {
            if _sroomid >= 0 {
                let sroomid = _sroomid.try_into().unwrap();
                let room = Room::init(sroomid, &config, &db, http_client.clone(), http_client2.clone()).await.expect("fetching room status error");
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
