use rand::{seq::IteratorRandom, thread_rng as rng};
use livekit_api::stream::{PlayInfo, PlayUrlCodec, PlayUrlCodecUrlInfo};

pub struct StreamInfo(PlayUrlCodec);
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
                                if codec.base_url != "" {
                                    matches!(codec.url_info, Some(_));
                                    return Some(StreamInfo(codec))
                                }
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
        let source = self.0.url_info.unwrap();
        let source = source.iter().filter(|source| !source.host.contains(".mcdn.")).choose(&mut rng()).unwrap();
        format!("{}{}{}", source.host, self.0.base_url, source.extra)
    }

    pub fn have_4k(&self) -> bool {
        self.0.accept_qn.contains(&20000)
    }
}
