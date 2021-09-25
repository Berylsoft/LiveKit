use futures_util::Stream;
use reqwest::get as http_get;
use crate::{
    config::GroupConfig,
    rest::room::{PlayInfo, PlayUrlCodec},
};

pub fn select_url(play_info: PlayInfo) -> Option<PlayUrlCodec> {
    for stream in play_info.playurl_info.playurl.stream {
        if "http_stream" == stream.protocol_name {
            for format in stream.format {
                if "flv" == format.format_name {
                    for codec in format.codec {
                        if "avc" == codec.codec_name {
                            return Some(codec)
                        }
                    }
                }
            }
        }
    }
    None
}

pub async fn get_url(roomid: u32, _group_config: &GroupConfig) -> Option<String> {
    let codec = select_url(PlayInfo::call(roomid, 10000).await.unwrap()).unwrap();
    for url_info in codec.url_info {
        if !url_info.host.contains(".mcdn.") {
            return Some(format!("{}{}{}", url_info.host, codec.base_url, url_info.extra))
        }
    }
    None
}

pub async fn get_stream(roomid: u32, group_config: &GroupConfig) -> impl Stream {
    let mut url = get_url(roomid, group_config).await.unwrap();

    loop {
        let resp = http_get(url).await.unwrap();
        match resp.status().as_u16() {
            200 => return resp.bytes_stream(),
            301 | 302 | 307 | 308 => url = resp.headers().get("location").unwrap().to_str().unwrap().to_string(),
            _ => panic!(),
        }
    }
}
