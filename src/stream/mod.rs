use futures::Stream;
use rand::{seq::IteratorRandom, thread_rng as rng};
use reqwest::get as http_get;
use crate::{
    config::GroupConfig,
    api::room::{PlayInfo, PlayUrlCodec, PlayUrlCodecUrlInfo},
};

pub type StreamInfo = PlayUrlCodec;
pub type StreamSourceInfo = PlayUrlCodecUrlInfo;

pub struct StreamType<'a> {
    // http_stream | http_hls
    pub protocol: &'a str,
    // flv | ts fmp4
    pub format: &'a str,
    // avc hevc
    pub codec: &'a str,
}

impl StreamType<'_> {
    pub fn select(&self, play_info: PlayInfo) -> Option<StreamInfo> {
        for stream in play_info.playurl_info.playurl.stream {
            if self.protocol == stream.protocol_name {
                for format in stream.format {
                    if self.format == format.format_name {
                        for codec in format.codec {
                            if self.codec == codec.codec_name {
                                return Some(codec)
                            }
                        }
                    }
                }
            }
        }
        None
    }
}

impl StreamInfo {
    pub fn to_url(self) -> String {
        let source = self.url_info.iter().filter(|source| !source.host.contains(".mcdn.")).choose(&mut rng()).unwrap();
        format!("{}{}{}", source.host, self.base_url, source.extra)
    }
}

pub async fn get_url(roomid: u32, _group_config: &GroupConfig) -> String {
    let stream_type = StreamType { protocol: "http_stream", format: "flv", codec: "avc" };
    let stream_info = stream_type.select(PlayInfo::call(roomid, 10000).await.unwrap()).unwrap();
    stream_info.to_url()
}

pub async fn get_stream(roomid: u32, group_config: &GroupConfig) -> impl Stream {
    let mut url = get_url(roomid, group_config).await;

    loop {
        let resp = http_get(url).await.unwrap();
        match resp.status().as_u16() {
            200 => return resp.bytes_stream(),
            301 | 302 | 307 | 308 => url = resp.headers().get("location").unwrap().to_str().unwrap().to_string(),
            _ => panic!(),
        }
    }
}
