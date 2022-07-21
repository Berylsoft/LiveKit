use std::path::PathBuf;
use structopt::StructOpt;
use tokio::{spawn, signal, time::{sleep, Duration}};
use livekit_api::client::HttpClient;
use livekit_feed::{config::*, storage::open_db};
use livekit_feedrec::rec;
use livekit_log_config::{log4rs, log_config};

#[derive(StructOpt)]
struct Args {
    #[structopt(short = "r", long)]
    roomid_list: String,
    #[structopt(short = "s", long, parse(from_os_str))]
    storage_path: PathBuf,
    #[structopt(short = "l", long, parse(from_os_str))]
    log_path: Option<PathBuf>,
    #[structopt(long)]
    log_debug: bool,
}

#[tokio::main]
async fn main() {
    let Args { roomid_list, storage_path, log_path, log_debug } = Args::from_args();
    if let Some(log_path) = log_path {
        log4rs::init_config(log_config(log_path, log_debug)).unwrap();
    }
    let db = open_db(storage_path).unwrap();
    let http_client = HttpClient::new_bare();
    for roomid in roomid_list.split(",").map(|roomid| roomid.parse::<u32>().unwrap()) {
        spawn(rec(roomid, &http_client, &db));
        sleep(Duration::from_millis(FEED_INIT_INTERVAL_MS)).await;
    }
    signal::ctrl_c().await.unwrap();
}
