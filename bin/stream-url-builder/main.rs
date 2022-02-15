use std::path::PathBuf;
use tokio::fs;
use structopt::StructOpt;
use livekit_api::{client::{HttpClient, Access}, stream::*};

#[derive(StructOpt)]
struct Args {
    #[structopt(short = "a", long, parse(from_os_str))]
    access_path: Option<PathBuf>,
    #[structopt(short = "r", long)]
    roomid: u32,
    #[structopt(short = "q", long)]
    qn: i32,
    #[structopt(short = "t", long)]
    triple: String,
}

fn select(triple: String, parsed: StreamInfo) -> Option<StreamKindInfo> {
    Some(match triple.as_str() {
        "flv-avc" => parsed.flv_avc,
        "flv-hevc" => parsed.flv_hevc?,
        "hls-ts-avc" => parsed.hls_ts_avc,
        "hls-ts-hevc" => parsed.hls_ts_hevc?,
        "hls-fmp4-avc" => parsed.hls_fmp4_avc?,
        "hls-fmp4-hevc" => parsed.hls_fmp4_hevc?,
        _ => return None,
    })
}

#[tokio::main]
async fn main() {
    let Args { access_path, roomid, qn, triple } = Args::from_args();
    let access = match access_path {
        Some(_path) => Some(serde_json::from_str::<Access>(fs::read_to_string(_path).await.unwrap().as_str()).unwrap()),
        None => None,
    };
    let client = HttpClient::new(access, None);
    let req = GetPlayInfo { roomid, qn: Qn(qn) };
    let resp: PlayInfo = client.call(&req).await.unwrap();
    let parsed = StreamInfo::parse(&resp.playurl_info.expect("stream is not on")).expect("parse error");
    let selected = select(triple, parsed).expect("selected kind does not exist");
    let url = selected.url(&selected.hosts[0]);
    println!("{}", url);
}
