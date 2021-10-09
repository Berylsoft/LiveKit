use futures::Stream;
use reqwest::get as http_get;
use crate::{
    api::room::PlayInfo,
    stream::url::StreamType,
};

pub async fn get_url(roomid: u32) -> String {
    let stream_type = StreamType { protocol: "http_stream", format: "flv", codec: "avc" };
    let stream_info = stream_type.select(PlayInfo::call(roomid, 10000).await.unwrap()).unwrap();
    stream_info.to_url()
}

pub async fn get_stream(roomid: u32) -> impl Stream {
    let mut url = get_url(roomid).await;

    loop {
        let resp = http_get(url).await.unwrap();
        match resp.status().as_u16() {
            200 => return resp.bytes_stream(),
            301 | 302 | 307 | 308 => url = resp.headers().get("location").unwrap().to_str().unwrap().to_string(),
            _ => panic!(),
        }
    }
}
