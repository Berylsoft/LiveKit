use structopt::StructOpt;
use tokio::signal;
use livekit::{config::STORAGE_VERSION, feed};

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
    for roomid in roomid_list.split(",").map(|roomid| roomid.parse::<u32>().unwrap()) {
        feed::client::init_record_only(roomid, format!("{}/{}-{}", storage_root, roomid, STORAGE_VERSION))
    }
    signal::ctrl_c().await.unwrap();
}
