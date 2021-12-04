use serde::Deserialize;
use serde_json::Value as JsonValue;
use crate::client::{HttpClient, RestApiResult};

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
    // pub uid: u32,
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

impl PlayInfo {
    #[inline]
    pub async fn call(client: &HttpClient, roomid: u32, qn: i32) -> RestApiResult<Self> {
        client.call(format!(
            "/xlive/web-room/v2/index/getRoomPlayInfo?room_id={}&protocol=0,1&format=0,1,2&codec=0,1&qn={}&platform=web&ptype=8",
            roomid,
            qn,
        )).await
    }
}
