use log::LevelFilter;
use log4rs::{
    append::file::FileAppender,
    config::{Appender, Config, Root},
    encode::json::JsonEncoder,
};
use structopt::StructOpt;
use tokio::{spawn, signal, time::{sleep, Duration}};
use livekit_api::client::HttpClient;
use livekit_feed::config::*;
use livekit_feed_storage::{open_db, rec};

pub fn log_config(path: String, debug: bool) -> Config {
    Config::builder()
        .appender(
            Appender::builder().build(
                "logfile",
                Box::new(
                    FileAppender::builder()
                        .encoder(Box::new(JsonEncoder::new()))
                        .build(path)
                        .unwrap(),
                ),
            ),
        )
        .build(
            Root::builder()
                .appender("logfile")
                .build(if debug { LevelFilter::Debug } else { LevelFilter::Info }),
        )
        .unwrap()
}

#[derive(StructOpt)]
struct Args {
    #[structopt(short = "r", long)]
    roomid_list: String,
    #[structopt(short = "s", long)]
    storage_path: String,
    #[structopt(short = "l", long)]
    log_path: Option<String>,
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
    let http_client = HttpClient::new_bare().await;
    for roomid in roomid_list.split(",").map(|roomid| roomid.parse::<u32>().unwrap()) {
        spawn(rec(roomid, &http_client, &db));
        sleep(Duration::from_millis(FEED_INIT_INTERVAL_MS)).await;
    }
    signal::ctrl_c().await.unwrap();
}
