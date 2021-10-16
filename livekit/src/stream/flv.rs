use tokio::{fs::File, io::AsyncWriteExt};
use futures::{Stream, StreamExt};
use reqwest::Client;
use crate::{
    api::room::PlayInfo,
    stream::url::StreamType,
};

pub async fn get_url(roomid: u32) -> String {
    let stream_type = StreamType { protocol: "http_stream", format: "flv", codec: "avc" };
    let stream_info = stream_type.select(PlayInfo::call(roomid, 10000).await.unwrap()).unwrap();
    stream_info.to_url()
}

const UA: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/94.0.4606.81 Safari/537.36";

pub async fn get_stream(url: String) -> Option<impl Stream<Item = Result<bytes::Bytes, reqwest::Error>>> {
    let mut url = url;
    let client = Client::builder().user_agent(UA).build().unwrap();
    loop {
        println!("{}", url.clone());
        let resp = client.get(url).header("Referer", "https://live.bilibili.com/").send().await.unwrap();
        match resp.status().as_u16() {
            200 => return Some(resp.bytes_stream()),
            404 => return None,
            301 | 302 | 307 | 308 => url = resp.headers().get("location").unwrap().to_str().unwrap().to_string(),
            status => panic!("{}", status),
        }
    }
}

pub async fn download(roomid: u32, path: String) {
    let url = get_url(roomid).await;
    let mut stream = get_stream(url).await.unwrap();
    let mut file = File::create(path).await.unwrap();
    loop {
        for data in stream.next().await {
            file.write(data.unwrap().as_ref()).await.unwrap();
        }
    }
}
