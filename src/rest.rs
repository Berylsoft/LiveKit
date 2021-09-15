use serde::Deserialize;

pub async fn call<Data>(url: String) -> Option<Data>
where
    Data: serde::de::DeserializeOwned,
{
    use serde_json::from_str as parse_json;
    use reqwest::{get as http_get, StatusCode};

    let resp = http_get(url.as_str()).await.unwrap();
    match resp.status() {
        StatusCode::OK => (),
        _ => return None,
    }
    let resp = resp.text().await.unwrap();
    let resp: RestApiResponse<Data> = parse_json(resp.as_str()).unwrap();
    match resp.code {
        0 => Some(resp.data),
        _ => None, // Err(resp.message)
    }
}

#[derive(Deserialize)]
pub struct RestApiResponse<Data> {
    pub code: i32,
    pub data: Data,
    pub message: String,
    pub ttl: i32,
}

#[derive(Deserialize)]
pub struct HostsInfo {
    pub host_list: Vec<HostInfo>,
    pub token: String,
}

#[derive(Deserialize)]
pub struct HostInfo {
    pub host: String,
    // pub port: u16,
    // pub ws_port: u16,
    pub wss_port: u16,
}

impl HostsInfo {
    #[inline]
    pub async fn get(roomid: u32) -> Option<Self> {
        call(format!(
            "https://api.live.bilibili.com/xlive/web-room/v1/index/getDanmuInfo?id={}&type=0",
            roomid
        )).await
    }
}