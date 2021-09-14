use serde::Deserialize;

#[derive(Deserialize)]
pub struct RestApiResponse<Data> {
    pub code: i32,
    pub data: Data,
    pub message: String,
    pub ttl: i32,
}

#[inline]
pub fn get_hosts_info(roomid: u32) -> String {
    format!(
        "https://api.live.bilibili.com/xlive/web-room/v1/index/getDanmuInfo?id={}&type=0",
        roomid
    )
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
