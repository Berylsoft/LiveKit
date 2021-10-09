use rand::{seq::IteratorRandom, thread_rng as rng};
use crate::{
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
