use tokio::{fs::File, io::AsyncWriteExt};
use futures::{Stream, StreamExt};
use livekit_api::{client::HttpClient, room::PlayInfo};
use crate::stream::url::StreamType;

pub async fn get_stream(client: &HttpClient, url: String) -> Option<impl Stream<Item = Result<bytes::Bytes, reqwest::Error>>> {
    let mut url = url;
    loop {
        println!("{}", url.clone());
        let resp = client.get(url).await.unwrap();
        match resp.status().as_u16() {
            200 => return Some(resp.bytes_stream()),
            404 => return None,
            301 | 302 | 307 | 308 => url = resp.headers().get("Location").unwrap().to_str().unwrap().to_owned(),
            status => panic!("{}", status),
        }
    }
}

pub async fn download(client: HttpClient, roomid: u32, path: String) {
    let stream_info = {
        let stream_type = StreamType { protocol: "http_stream", format: "flv", codec: "avc" };
        let stream_info = stream_type.select(PlayInfo::call(&client, roomid, 10000).await.unwrap()).unwrap();
        if stream_info.have_4k() {
            stream_type.select(PlayInfo::call(&client, roomid, 20000).await.unwrap()).unwrap()
        } else {
            stream_info
        }
    };
    let mut stream = get_stream(&client, stream_info.to_url()).await.unwrap();
    let mut file = File::create(path).await.unwrap();
    loop {
        while let Some(data) = stream.next().await {
            file.write(data.unwrap().as_ref()).await.unwrap();
        }
    }
}
