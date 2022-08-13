use serde::{Serialize, Deserialize};
use serde_json::Value as JsonValue;
use rand::{seq::IteratorRandom, thread_rng as rng};
use crate::client::{RestApi, RestApiRequestKind};

/*

type Quality = i32;

#[derive(Deserialize)]
pub struct PlayUrlCodecUrlInfo {
    pub host: String,
    pub extra: String,
    pub stream_ttl: i32,
}

#[derive(Deserialize)]
pub struct PlayUrlCodec {
    pub codec_name: String,
    pub current_qn: Quality,
    pub accept_qn: Vec<Quality>,
    pub base_url: String,
    pub url_info: Option<Vec<PlayUrlCodecUrlInfo>>,
    // pub hdr_qn: Option<Quality>,
}

#[derive(Deserialize)]
pub struct PlayUrlFormat {
    pub format_name: String,
    pub codec: Vec<PlayUrlCodec>,
}

#[derive(Deserialize)]
pub struct PlayUrlStream {
    pub protocol_name: String,
    pub format: Vec<PlayUrlFormat>,
}

// #[derive(Deserialize)]
// pub struct PlayUrlP2PData {
//     pub p2p: bool,
//     pub p2p_type: _num,
//     pub m_p2p: bool,
//     pub m_servers: Vec<String>,
// }

#[derive(Deserialize)]
pub struct PlayUrl {
    // pub cid: u32, // roomid
    // pub g_qn_desc: Vec<_>,
    pub stream: Vec<PlayUrlStream>,
    // pub p2p_data: Option<PlayUrlP2PData>,
    // pub dolby_qn: Option<Quality>,
}

#[derive(Deserialize)]
pub struct PlayUrlInfo {
    // pub conf_json: String,
    pub playurl: PlayUrl,
}

*/

#[derive(Deserialize)]
pub struct PlayInfo {
    // pub room_id: u32,
    // pub short_id: u32,
    // pub uid: u64,
    // pub is_hidden: bool,
    // pub is_locked: bool,
    // pub is_portrait: bool,
    // pub live_status: u8,
    // pub hidden_till: _num,
    // pub lock_till: _num,
    // pub encrypted: bool,
    // pub pwd_verified: bool,
    // pub live_time: u64,
    // pub room_shield: _num,
    // pub all_special_types: Vec<_>,
    pub playurl_info: Option<JsonValue>,
}

#[derive(Serialize)]
pub struct GetPlayInfo {
    pub roomid: u32,
    pub qn: Qn,
}

impl RestApi for GetPlayInfo {
    type Response = PlayInfo;

    fn kind(&self) -> RestApiRequestKind {
        RestApiRequestKind::Get
    }

    fn path(&self) -> String {
        format!(
            "/xlive/web-room/v2/index/getRoomPlayInfo?room_id={}&protocol=0,1&format=0,1,2&codec=0,1&qn={}&platform=web&ptype=8",
            self.roomid,
            self.qn.0,
        )
    }
}

fn to<D: serde::de::DeserializeOwned>(value: &JsonValue) -> Option<D>
{
    D::deserialize(value).ok()
}

#[derive(Debug, PartialEq, Serialize)]
pub struct Qn(pub i32);

impl From<i32> for Qn {
    fn from(n: i32) -> Qn {
        Qn(n)
    }
}

impl From<Qn> for i32 {
    fn from(qn: Qn) -> i32 {
        qn.0
    }
}

impl<'a> From<Qn> for &'a str {
    fn from(qn: Qn) -> &'a str {
        match qn.0 {
            20000 => "4K",
            10000 => "原画",
            401 => "蓝光(杜比)",
            400 => "蓝光",
            250 => "超清",
            150 => "高清",
            80 => "流畅",
            _ => "<unknown>",
        }
    }
}

impl Qn {
    pub fn has_hdr(&self) -> bool {
        match self.0 {
            400 | 250 => true,
            _ => false,
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct StreamInfo {
    pub flv_avc: StreamKindInfo,
    pub flv_hevc: Option<StreamKindInfo>,
    pub hls_ts_avc: StreamKindInfo,
    pub hls_ts_hevc: Option<StreamKindInfo>,
    pub hls_fmp4_avc: Option<StreamKindInfo>,
    pub hls_fmp4_hevc: Option<StreamKindInfo>,
}

impl StreamInfo {
    pub fn parse(raw: &JsonValue) -> Option<StreamInfo> {
        macro_rules! filter_impl {
            ($list:expr, $kindofkind1:literal, $kindofkind2:literal, $kind:literal) => {
                $list.iter()
                    .filter(|el| el[$kindofkind1] == $kind)
                    .flat_map(|el| el[$kindofkind2].as_array().unwrap())
                    .collect::<Vec<&JsonValue>>()
            };
        }

        macro_rules! filter_impl2 {
            ($list:expr, $kindofkind1:literal, $kind:literal) => {
                $list.iter()
                    .filter(|el| el[$kindofkind1] == $kind)
                    .collect::<Vec<&&JsonValue>>()
            };
        }

        macro_rules! layer1 {
            ($list:expr, $kind:literal) => {
                filter_impl!($list, "protocol_name", "format", $kind)
            };
        }

        macro_rules! layer2 {
            ($list:expr, $kind:literal) => {
                filter_impl!($list, "format_name", "codec", $kind)
            };
        }

        macro_rules! layer3 {
            ($list:expr, $kind:literal) => {
                StreamKindInfo::from_list(filter_impl2!($list, "codec_name", $kind))
            };
        }

        let raw = raw["playurl"]["stream"].as_array()?;

        let stream = layer1!(raw, "http_stream");
        let hls = layer1!(raw, "http_hls");

        let flv = layer2!(stream, "flv");
        let hls_ts = layer2!(hls, "ts");
        let hls_fmp4 = layer2!(hls, "fmp4");

        Some(StreamInfo {
            flv_avc: layer3!(flv, "avc")?,
            flv_hevc: layer3!(flv, "hevc"),
            hls_ts_avc: layer3!(hls_ts, "avc")?,
            hls_ts_hevc: layer3!(hls_ts, "hevc"),
            hls_fmp4_avc: layer3!(hls_fmp4, "avc"),
            hls_fmp4_hevc: layer3!(hls_fmp4, "hevc"),
        })
    }
}

#[derive(Debug, PartialEq)]
pub struct StreamKindInfo {
    pub current_qn: Qn,
    pub accept_qn: Vec<Qn>,
    // pub hdr_qn: Option<Qn>,
    pub base_url: String,
    pub hosts: Vec<HostInfo>,
}

impl StreamKindInfo {
    pub fn from_list(list: Vec<&&JsonValue>) -> Option<StreamKindInfo> {
        let mut x = list.into_iter();
        let n = x.next()?;
        if let Some(_) = x.next() {
            return None
        }
        StreamKindInfo::parse(n)
    }

    pub fn parse(raw: &JsonValue) -> Option<StreamKindInfo> {
        let base_url: String = to(&raw["base_url"])?;
        if base_url.is_empty() | raw["url_info"].is_null() { return None };
        Some(StreamKindInfo {
            current_qn: Qn(to(&raw["current_qn"])?),
            accept_qn: to::<Vec<i32>>(&raw["accept_qn"])?.into_iter().map(|qn| Qn(qn)).collect(),
            hosts: to(&raw["url_info"])?,
            base_url,
        })
    }
}

#[derive(Deserialize, Debug, PartialEq)]
pub struct HostInfo {
    pub host: String,
    pub extra: String,
    pub stream_ttl: i32,
}

impl StreamKindInfo {
    pub fn url(&self, host: &HostInfo) -> String {
        format!("{}{}{}", host.host, self.base_url, host.extra)
    }

    pub fn rand_url(&self) -> String {
        let host = self.hosts.iter().filter(|source| !source.host.contains(".mcdn.")).choose(&mut rng()).unwrap();
        self.url(host)
    }

    pub fn have_qn(&self, qn: Qn) -> bool {
        self.accept_qn.contains(&qn)
    }

    pub fn have_4k(&self) -> bool {
        self.have_qn(Qn(20000))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! parse {
        ($resptext:literal) => {
            StreamInfo::parse(&serde_json::from_str::<JsonValue>($resptext).unwrap()["data"]["playurl_info"])
        };
    }

    macro_rules! some {
        ($result:literal, $resptext:literal) => {
            assert_eq!($result, format!("{:?}", parse!($resptext).unwrap()))
        };
    }

    macro_rules! none {
        ($resptext:literal) => {
            assert_eq!(None, parse!($resptext))
        };
    }

    #[test]
    fn happypath() {
        some!(r#"StreamInfo { flv_avc: StreamKindInfo { current_qn: Qn(10000), accept_qn: [Qn(10000), Qn(400), Qn(250), Qn(150), Qn(80)], base_url: "/live-bvc/794141/live_745493_5673110.flv?expires=1638624779&len=0&oi=3742161066&pt=web&qn=10000&trid=1000b5e2900f554b47e9ac0de4e7750e2962&sigparams=cdn,expires,len,oi,pt,qn,trid", hosts: [HostInfo { host: "https://d1--cn-gotcha04.bilivideo.com", extra: "&cdn=cn-gotcha04&sign=4a6e8cfcd48ba38740b624cf31deb036&p2p_type=0&src=9&sl=6&free_type=0&flowtype=1&machinezone=ylf&sk=c903950cdaa05f204444c464c17af80e&source=onetier&order=1", stream_ttl: 3600 }] }, flv_hevc: None, hls_ts_avc: StreamKindInfo { current_qn: Qn(10000), accept_qn: [Qn(10000), Qn(400), Qn(250), Qn(150), Qn(80)], base_url: "/live-bvc/794141/live_745493_5673110.m3u8?expires=1638624779&len=0&oi=3742161066&pt=web&qn=10000&trid=1003b5e2900f554b47e9ac0de4e7750e2962&sigparams=cdn,expires,len,oi,pt,qn,trid", hosts: [HostInfo { host: "https://d1--cn-gotcha103.bilivideo.com", extra: "&cdn=cn-gotcha03&sign=6909557fd7cb11ae4d254c00de8ec904&p2p_type=0&src=9&sl=6&free_type=0&flowtype=1&machinezone=ylf&sk=c903950cdaa05f204444c464c17af80e&source=onetier&order=1", stream_ttl: 3600 }] }, hls_ts_hevc: None, hls_fmp4_avc: Some(StreamKindInfo { current_qn: Qn(10000), accept_qn: [Qn(10000), Qn(400), Qn(250), Qn(150), Qn(80)], base_url: "/live-bvc/794141/live_745493_5673110/index.m3u8?expires=1638624779&len=0&oi=3742161066&pt=web&qn=10000&trid=1007b5e2900f554b47e9ac0de4e7750e2962&sigparams=cdn,expires,len,oi,pt,qn,trid", hosts: [HostInfo { host: "https://d1--cn-gotcha208.bilivideo.com", extra: "&cdn=cn-gotcha08&sign=4c4db95b6a128358a503016a1989f07e&p2p_type=0&src=9&sl=6&free_type=0&flowtype=1&machinezone=ylf&sk=c9c6154426932efa80d25af02e87a3bd&source=onetier&order=1", stream_ttl: 3600 }, HostInfo { host: "https://d1--cn-gotcha204.bilivideo.com", extra: "&cdn=cn-gotcha04&sign=c93ac7aa570e64c9faf1055036353d42&p2p_type=0&src=9&sl=6&free_type=0&flowtype=1&machinezone=ylf&sk=c9c6154426932efa80d25af02e87a3bd&source=onetier&order=2", stream_ttl: 3600 }, HostInfo { host: "https://d1--cn-gotcha203.bilivideo.com", extra: "&cdn=cn-gotcha03&sign=b4268da55de2dbe0084af4a2a5ba2631&p2p_type=0&src=9&sl=6&free_type=0&flowtype=1&machinezone=ylf&sk=c9c6154426932efa80d25af02e87a3bd&source=onetier&order=3", stream_ttl: 3600 }, HostInfo { host: "https://d1--cn-gotcha202.bilivideo.com", extra: "&cdn=cn-gotcha02&sign=c0300adccd6cb5bc9cfa857949623ba1&p2p_type=0&src=9&sl=6&free_type=0&flowtype=1&machinezone=ylf&sk=c9c6154426932efa80d25af02e87a3bd&source=onetier&order=4", stream_ttl: 3600 }] }), hls_fmp4_hevc: None }"#, r#"{"code":0,"message":"0","ttl":1,"data":{"room_id":8792912,"short_id":0,"uid":745493,"is_hidden":false,"is_locked":false,"is_portrait":false,"live_status":1,"hidden_till":0,"lock_till":0,"encrypted":false,"pwd_verified":true,"live_time":1638615261,"room_shield":1,"all_special_types":[],"playurl_info":{"conf_json":"{\"cdn_rate\":10000,\"report_interval_sec\":150}","playurl":{"cid":8792912,"g_qn_desc":[{"qn":20000,"desc":"4K","hdr_desc":""},{"qn":10000,"desc":"原画","hdr_desc":""},{"qn":401,"desc":"蓝光(杜比)","hdr_desc":""},{"qn":400,"desc":"蓝光","hdr_desc":"HDR"},{"qn":250,"desc":"超清","hdr_desc":"HDR"},{"qn":150,"desc":"高清","hdr_desc":""},{"qn":80,"desc":"流畅","hdr_desc":""}],"stream":[{"protocol_name":"http_stream","format":[{"format_name":"flv","codec":[{"codec_name":"avc","current_qn":10000,"accept_qn":[10000,400,250,150,80],"base_url":"/live-bvc/794141/live_745493_5673110.flv?expires=1638624779\u0026len=0\u0026oi=3742161066\u0026pt=web\u0026qn=10000\u0026trid=1000b5e2900f554b47e9ac0de4e7750e2962\u0026sigparams=cdn,expires,len,oi,pt,qn,trid","url_info":[{"host":"https://d1--cn-gotcha04.bilivideo.com","extra":"\u0026cdn=cn-gotcha04\u0026sign=4a6e8cfcd48ba38740b624cf31deb036\u0026p2p_type=0\u0026src=9\u0026sl=6\u0026free_type=0\u0026flowtype=1\u0026machinezone=ylf\u0026sk=c903950cdaa05f204444c464c17af80e\u0026source=onetier\u0026order=1","stream_ttl":3600}],"hdr_qn":null},{"codec_name":"hevc","current_qn":0,"accept_qn":[400,250],"base_url":"","url_info":null,"hdr_qn":null}]}]},{"protocol_name":"http_hls","format":[{"format_name":"ts","codec":[{"codec_name":"avc","current_qn":10000,"accept_qn":[10000,400,250,150,80],"base_url":"/live-bvc/794141/live_745493_5673110.m3u8?expires=1638624779\u0026len=0\u0026oi=3742161066\u0026pt=web\u0026qn=10000\u0026trid=1003b5e2900f554b47e9ac0de4e7750e2962\u0026sigparams=cdn,expires,len,oi,pt,qn,trid","url_info":[{"host":"https://d1--cn-gotcha103.bilivideo.com","extra":"\u0026cdn=cn-gotcha03\u0026sign=6909557fd7cb11ae4d254c00de8ec904\u0026p2p_type=0\u0026src=9\u0026sl=6\u0026free_type=0\u0026flowtype=1\u0026machinezone=ylf\u0026sk=c903950cdaa05f204444c464c17af80e\u0026source=onetier\u0026order=1","stream_ttl":3600}],"hdr_qn":null}]},{"format_name":"fmp4","codec":[{"codec_name":"avc","current_qn":10000,"accept_qn":[10000,400,250,150,80],"base_url":"/live-bvc/794141/live_745493_5673110/index.m3u8?expires=1638624779\u0026len=0\u0026oi=3742161066\u0026pt=web\u0026qn=10000\u0026trid=1007b5e2900f554b47e9ac0de4e7750e2962\u0026sigparams=cdn,expires,len,oi,pt,qn,trid","url_info":[{"host":"https://d1--cn-gotcha208.bilivideo.com","extra":"\u0026cdn=cn-gotcha08\u0026sign=4c4db95b6a128358a503016a1989f07e\u0026p2p_type=0\u0026src=9\u0026sl=6\u0026free_type=0\u0026flowtype=1\u0026machinezone=ylf\u0026sk=c9c6154426932efa80d25af02e87a3bd\u0026source=onetier\u0026order=1","stream_ttl":3600},{"host":"https://d1--cn-gotcha204.bilivideo.com","extra":"\u0026cdn=cn-gotcha04\u0026sign=c93ac7aa570e64c9faf1055036353d42\u0026p2p_type=0\u0026src=9\u0026sl=6\u0026free_type=0\u0026flowtype=1\u0026machinezone=ylf\u0026sk=c9c6154426932efa80d25af02e87a3bd\u0026source=onetier\u0026order=2","stream_ttl":3600},{"host":"https://d1--cn-gotcha203.bilivideo.com","extra":"\u0026cdn=cn-gotcha03\u0026sign=b4268da55de2dbe0084af4a2a5ba2631\u0026p2p_type=0\u0026src=9\u0026sl=6\u0026free_type=0\u0026flowtype=1\u0026machinezone=ylf\u0026sk=c9c6154426932efa80d25af02e87a3bd\u0026source=onetier\u0026order=3","stream_ttl":3600},{"host":"https://d1--cn-gotcha202.bilivideo.com","extra":"\u0026cdn=cn-gotcha02\u0026sign=c0300adccd6cb5bc9cfa857949623ba1\u0026p2p_type=0\u0026src=9\u0026sl=6\u0026free_type=0\u0026flowtype=1\u0026machinezone=ylf\u0026sk=c9c6154426932efa80d25af02e87a3bd\u0026source=onetier\u0026order=4","stream_ttl":3600}],"hdr_qn":null},{"codec_name":"hevc","current_qn":0,"accept_qn":[400,250],"base_url":"","url_info":null,"hdr_qn":null}]}]}],"p2p_data":{"p2p":false,"p2p_type":0,"m_p2p":true,"m_servers":["https://xy59x45x75x29xy.mcdn.bilivideo.cn:486"]},"dolby_qn":null}}}}"#)
    }

    #[test]
    fn playurl_info_null() {
        none!(r#"{"code":0,"message":"0","ttl":1,"data":{"room_id":22714455,"short_id":0,"uid":1734305780,"is_hidden":false,"is_locked":false,"is_portrait":false,"live_status":0,"hidden_till":0,"lock_till":0,"encrypted":false,"pwd_verified":true,"live_time":0,"room_shield":0,"all_special_types":[],"playurl_info":null}}"#)
    }
}
