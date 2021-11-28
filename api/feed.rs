use serde::Deserialize;
use crate::client::{HttpClient, RestApiResult};

#[derive(Deserialize)]
pub struct HostsInfo {
    pub host_list: Vec<HostInfo>,
    pub token: String,
}

#[derive(Deserialize)]
pub struct HostInfo {
    pub host: String,
    pub port: u16,
    pub ws_port: u16,
    pub wss_port: u16,
}

// TODO (consider) macro
impl HostsInfo {
    #[inline]
    pub async fn call(client: &HttpClient, roomid: u32) -> RestApiResult<Self> {
        client.call(format!(
            "/xlive/web-room/v1/index/getDanmuInfo?id={}&type=0",
            roomid
        )).await
    }
}
