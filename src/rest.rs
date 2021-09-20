use serde::Deserialize;

pub async fn call<Data>(url: String) -> Result<Data, String>
where
    Data: serde::de::DeserializeOwned,
{
    use reqwest::{get as http_get, StatusCode};

    // TODO error handle
    let resp = http_get(url.as_str()).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let resp = resp.text().await.unwrap();
    let resp: RestApiResponse<Data> = serde_json::from_str(resp.as_str()).unwrap();
    match resp.code {
        0 => Ok(resp.data),
        _ => Err(resp.message),
    }
}

#[derive(Deserialize)]
pub struct RestApiResponse<Data> {
    pub code: i32,
    pub data: Data,
    pub message: String,
    pub ttl: i32,
}

pub mod room {
    use super::{call, Deserialize};

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

    // TODO (consider) macro
    impl HostsInfo {
        #[inline]
        pub async fn call(roomid: u32) -> Result<Self, String> {
            call(format!(
                "https://api.live.bilibili.com/xlive/web-room/v1/index/getDanmuInfo?id={}&type=0",
                roomid
            )).await
        }
    }

    #[derive(Deserialize)]
    #[repr(u8)]
    pub enum LiveStatus {
        Off = 0,
        On,
        Carousel,
    }

    #[derive(Deserialize)]
    pub struct RoomInfo {
        pub uid: u32,
        pub room_id: u32,
        pub short_id: u32,
        pub live_status: LiveStatus,
        pub parent_area_name: String,
        pub area_name: String,
        pub title: String,
        // pub attention: u32,
        // pub online: u32,
        // pub is_portrait: bool,
        // pub description: String,
        // pub area_id: u16,
        // pub parent_area_id: u8,
        // pub background: String,
        // pub user_cover: String,
        // pub keyframe: String,
        // pub tags: String,
    }

    impl RoomInfo {
        #[inline]
        pub async fn call(roomid: u32) -> Result<Self, String> {
            call(format!(
                "https://api.live.bilibili.com/room/v1/Room/get_info?id={}",
                roomid
            )).await
        }
    }
}
