use std::path::PathBuf;
use tokio::{fs::File, io::AsyncWriteExt};
use futures::{Stream, StreamExt};
use livekit_api::{client::{HttpClient, ReqwestError}, stream::{PlayInfo, StreamInfo}};

pub async fn get_stream(client: &HttpClient, url: String) -> Option<impl Stream<Item = Result<bytes::Bytes, ReqwestError>>> {
    println!("{}", url);
    let resp = client.get(url).await.unwrap();
    match resp.status().as_u16() {
        200 => return Some(resp.bytes_stream()),
        404 => return None,
        301 | 302 | 307 | 308 => panic!("{}", resp.headers().get("Location").unwrap().to_str().unwrap()),
        status => panic!("{}", status),
    }
}

pub async fn download(client: HttpClient, roomid: u32, path: PathBuf) {
    macro_rules! x {
        ($qn:expr) => {
            StreamInfo::parse(&PlayInfo::call(&client, roomid, $qn).await.unwrap().playurl_info.unwrap()).unwrap().flv_avc
        };
    }
    let stream_info = {
        let stream_info = x!(10000);
        if stream_info.have_4k() {
            x!(20000)
        } else {
            stream_info
        }
    };
    let mut stream = get_stream(&client, stream_info.rand_url()).await.unwrap();
    let mut file = File::create(path).await.unwrap();
    while let Some(data) = stream.next().await {
        file.write(data.unwrap().as_ref()).await.unwrap();
    }
}
