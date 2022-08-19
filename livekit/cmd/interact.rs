use std::{path::PathBuf, fs};
use bilibili_restapi_live::interact as api;
use bilibili_restapi_client::{client::Client, access::Access};
use livekit_feed::stream::now;

/// call interact apis with json from command line
#[derive(argh::FromArgs)]
#[argh(subcommand, name = "interact")]
pub struct Args {
    /// json access file path of bilibili_restapi
    #[argh(option, short = 'a')]
    access_path: PathBuf,
    #[argh(subcommand)]
    request: Request,
}

#[derive(argh::FromArgs)]
#[argh(subcommand)]
enum Request {
    Danmaku(Danmaku),
}

/// send a common danmaku to specified room
#[derive(argh::FromArgs)]
#[argh(subcommand, name = "send-dm")]
struct Danmaku {
    /// target roomid (no short id)
    #[argh(option, short = 'r')]
    roomid: u32,
    /// danmaku content
    #[argh(option, short = 't')]
    text: String,
    /// customize the rand value
    #[argh(option)]
    rand: Option<i64>,
    /// emoji danmaku
    #[argh(switch)]
    emoji: bool,
}

pub async fn main(Args { access_path, request }: Args) {
    let access: Access = serde_json::from_reader(fs::OpenOptions::new().read(true).open(access_path).unwrap()).unwrap();
    let client = Client::new(Some(access), None);
    match request {
        Request::Danmaku(Danmaku { roomid, text: msg, rand: rnd, emoji }) => {
            let rnd = rnd.unwrap_or_else(|| now().try_into().unwrap());
            let send = api::SendDanmaku::new(roomid, msg, rnd, emoji);
            println!("{:?}", client.call(&send).await);
        }
    }
}
