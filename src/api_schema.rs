pub mod rest {
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
}

pub mod msg {
    use serde::Serialize;

    #[derive(Serialize)]
    pub struct ConnectInfo {
        pub uid: u32,
        pub roomid: u32,
        pub protover: u8, // unknown number
        pub platform: String,
        pub r#type: u8, // unknown number
        pub key: Option<String>,
    }
}
