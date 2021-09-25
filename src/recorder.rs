use crate::rest::room::{PlayInfo, PlayUrlCodec};

fn select_url(play_info: PlayInfo) -> Option<PlayUrlCodec> {
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

pub async fn init(roomid: u32) -> Option<String> {
    let codec = select_url(PlayInfo::call(roomid, 10000).await.unwrap()).unwrap();
    for url_info in codec.url_info {
        if !url_info.host.contains(".mcdn.") {
            return Some(format!("{}{}{}", url_info.host, codec.base_url, url_info.extra))
        }
    }
    None
}
