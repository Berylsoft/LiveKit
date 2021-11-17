use log::LevelFilter;
use log4rs::{
    append::file::FileAppender,
    config::{Appender, Config, Root},
    encode::json::JsonEncoder,
};
use structopt::StructOpt;
use tokio::{spawn, signal, time::{sleep, Duration}};
use livekit::{
    config::FEED_INIT_INTERVAL_MILLISEC,
    util::http::HttpClient,
    feed::client::client_rec,
};

pub fn log_config(path: String) -> Config {
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
                .build(LevelFilter::Debug),
        )
        .unwrap()
}

#[derive(StructOpt)]
struct Args {
    #[structopt(short = "r", long)]
    roomid_list: String,
    #[structopt(short = "s", long)]
    storage_root: String,
    #[structopt(short = "l", long)]
    log_path: Option<String>,
}

#[tokio::main]
async fn main() {
    let Args { roomid_list, storage_root, log_path } = Args::from_args();
    if let Some(log_path) = log_path {
        log4rs::init_config(log_config(log_path)).unwrap();
    }
    let db = sled::open(storage_root).unwrap();
    let http_client = HttpClient::new_bare().await;
    for roomid in roomid_list.split(",").map(|roomid| roomid.parse::<u32>().unwrap()) {
        let storage = db.open_tree(roomid.to_string()).unwrap();
        spawn(client_rec(roomid, http_client.clone(), storage));
        sleep(Duration::from_millis(FEED_INIT_INTERVAL_MILLISEC)).await;
    }
    signal::ctrl_c().await.unwrap();
}
