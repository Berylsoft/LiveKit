use std::sync::Arc;
use tokio::{fs::File, io::AsyncWriteExt};
use futures::{Stream, StreamExt};
use crate::{
    util::http::HttpClient,
    api::room::PlayInfo,
    stream::url::StreamType,
};

pub async fn get_stream(client: &HttpClient, url: String) -> Option<impl Stream<Item = Result<bytes::Bytes, reqwest::Error>>> {
    let mut url = url;
    loop {
        println!("{}", url.clone());
        let resp = client.get(url).await;
        match resp.status().as_u16() {
            200 => return Some(resp.bytes_stream()),
            404 => return None,
            301 | 302 | 307 | 308 => url = resp.headers().get("Location").unwrap().to_str().unwrap().to_string(),
            status => panic!("{}", status),
        }
    }
}

pub async fn download(client: Arc<HttpClient>, roomid: u32, path: String) {
    let stream_type = StreamType { protocol: "http_stream", format: "flv", codec: "avc" };
    let stream_info = stream_type.select(PlayInfo::call(&client, roomid, 10000).await.unwrap()).unwrap();
    let mut stream = get_stream(&client, stream_info.to_url()).await.unwrap();
    let mut file = File::create(path).await.unwrap();
    loop {
        for data in stream.next().await {
            file.write(data.unwrap().as_ref()).await.unwrap();
        }
    }
}
