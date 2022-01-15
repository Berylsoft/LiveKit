use serde::{Serialize, Deserialize};
use crate::client::{HttpClient, RestApiResult};
use crate::client::{RestApi, RestApiRequestKind};

#[derive(Serialize)]
pub struct GetHostsInfo {
    pub roomid: u32,
}

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

impl HostsInfo {
    #[inline]
    pub async fn call(client: &HttpClient, roomid: u32) -> RestApiResult<Self> {
        client.call(format!(
            "/xlive/web-room/v1/index/getDanmuInfo?id={}&type=0",
            roomid
        )).await
    }
}

impl RestApi for GetHostsInfo {
    type Response = HostsInfo;

    fn kind(&self) -> RestApiRequestKind {
        RestApiRequestKind::Get
    }

    fn path(&self) -> String {
        format!(
            "/xlive/web-room/v1/index/getDanmuInfo?id={}&type=0",
            self.roomid
        )
    }
}
