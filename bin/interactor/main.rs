use structopt::StructOpt;
use livekit_api::{client::{HttpClient, Access}, interact::SendDanmaku};

#[derive(StructOpt)]
struct Args {
    #[structopt(short = "a", long)]
    access: String,
    #[structopt(short = "p", long)]
    payload: String,
}

#[derive(serde::Deserialize)]
#[serde(tag = "type", content = "data")]
enum Payload {
    Danmaku { roomid: u32, msg: String, rnd: Option<i64>, emoji: bool },
}

#[tokio::main]
async fn main() {
    let Args { access, payload } = Args::from_args();
    let access: Access = serde_json::from_str(tokio::fs::read_to_string(access).await.unwrap().as_str()).unwrap();
    let payload: Payload = serde_json::from_str(payload.as_str()).unwrap();
    let client = HttpClient::new(Some(access), None).await;
    match payload {
        Payload::Danmaku { roomid, msg, rnd, emoji } => {
            let rnd = match rnd {
                Some(rnd) => rnd,
                None => {
                    use std::time::{SystemTime, UNIX_EPOCH};
                    SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis().try_into().unwrap()
                }
            };
            let send = SendDanmaku::new(roomid, msg, rnd, emoji);
            println!("{:?}", send.call(&client).await);
        }
    }
}