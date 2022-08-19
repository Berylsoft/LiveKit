use std::{path::PathBuf, fs};
use serde::Deserialize;
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
    /// json request
    #[argh(option, short = 'r')]
    request: String,
}

#[derive(Deserialize)]
#[serde(tag = "type", content = "data")]
enum Request {
    Danmaku { roomid: u32, msg: String, rnd: Option<i64>, emoji: bool },
}

pub async fn main(Args { access_path, request }: Args) {
    let access: Access = serde_json::from_reader(fs::OpenOptions::new().read(true).open(access_path).unwrap()).unwrap();
    let payload: Request = serde_json::from_str(&request).unwrap();
    let client = Client::new(Some(access), None);
    match payload {
        Request::Danmaku { roomid, msg, rnd, emoji } => {
            let rnd = rnd.unwrap_or_else(|| now().try_into().unwrap());
            let send = api::SendDanmaku::new(roomid, msg, rnd, emoji);
            println!("{:?}", client.call(&send).await);
        }
    }
}
