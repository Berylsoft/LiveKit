use structopt::StructOpt;
use std::sync::Arc;
use tokio::{spawn, signal, time::{sleep, Duration}};
use livekit::{
    config::{STORAGE_VERSION, FEED_INIT_INTERVAL_MILLISEC},
    util::http::HttpClient,
    feed::client::{open_storage, client_rec}
};

#[derive(StructOpt)]
struct Args {
    #[structopt(short = "p", long)]
    storage_root: String,
    #[structopt(short = "r", long)]
    roomid_list: String,
}

#[tokio::main]
async fn main() {
    let Args { storage_root, roomid_list } = Args::from_args();
    let http_client = Arc::new(HttpClient::new_bare().await);
    for roomid in roomid_list.split(",").map(|roomid| roomid.parse::<u32>().unwrap()) {
        let storage = open_storage(format!("{}/{}-{}", storage_root, roomid, STORAGE_VERSION)).unwrap();
        spawn(client_rec(roomid, http_client.clone(), storage));
        sleep(Duration::from_millis(FEED_INIT_INTERVAL_MILLISEC)).await;
    }
    signal::ctrl_c().await.unwrap();
}
