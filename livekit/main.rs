use std::path::PathBuf;
use structopt::StructOpt;
use tokio::{signal, fs};
use livekit_api::client::HttpClient;
use livekit::{config::*, room::Group};

#[derive(StructOpt)]
struct Args {
    #[structopt(short = "c", long, parse(from_os_str))]
    config_path: PathBuf,
}

#[tokio::main]
async fn main() {
    env_logger::init();

    let config = fs::read_to_string(Args::from_args().config_path).await.expect("loading config error");
    let GlobalConfig { group } = toml::from_str(config.as_str()).expect("parsing config error");

    let http_client2 = HttpClient::new_bare().await;

    for GroupConfig { config, rooms } in group {
        let group = Group::init(config, &http_client2).await;
        for msroomid in rooms {
            group.spawn(msroomid).await;
        }
    }

    signal::ctrl_c().await.unwrap();
}
