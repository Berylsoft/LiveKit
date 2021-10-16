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

pub async fn get_stream(url: String) -> Option<impl Stream<Item = Result<bytes::Bytes, reqwest::Error>>> {
    let mut url = url;
    loop {
        println!("{}", url);
        let resp = http_get(url).await.unwrap();
        match resp.status().as_u16() {
            200 => return Some(resp.bytes_stream()),
            404 => return None,
            301 | 302 | 307 | 308 => url = resp.headers().get("location").unwrap().to_str().unwrap().to_string(),
            status => panic!("{}", status),
        }
    }
}
