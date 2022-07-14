use std::path::PathBuf;
use log::LevelFilter;
use log4rs::{
    append::file::FileAppender,
    config::{Appender, Config, Root},
    encode::json::JsonEncoder,
};
use structopt::StructOpt;
use tokio::{spawn, signal, net::TcpListener};
use livekit_api::client::HttpClient;
use livekit_feed::{storage::open_db};
use livekit_feed_reprod::{rec, conn};

pub fn log_config(path: PathBuf, debug: bool) -> Config {
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
    roomid: u32,
    #[structopt(short = "p", long)]
    port: u16,
    #[structopt(short = "s", long, parse(from_os_str))]
    storage_path: PathBuf,
    #[structopt(short = "l", long, parse(from_os_str))]
    log_path: Option<PathBuf>,
    #[structopt(long)]
    log_debug: bool,
}

#[tokio::main]
async fn main() {
    let Args { roomid, port, storage_path, log_path, log_debug } = Args::from_args();
    if let Some(log_path) = log_path {
        log4rs::init_config(log_config(log_path, log_debug)).unwrap();
    }
    let db = open_db(storage_path).unwrap();
    let http_client = HttpClient::new_bare();
    let (thread, event_rx) = rec(roomid, &http_client, &db);
    spawn(thread);
    let socket = TcpListener::bind(("0.0.0.0", port)).await.unwrap();
    while let Ok((stream, _)) = socket.accept().await {
        tokio::spawn(conn(stream, event_rx.clone()));
    }
    signal::ctrl_c().await.unwrap();
}
