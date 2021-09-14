pub enum Package {
    Unknown(Vec<u8>),
    InitRequest(String),
    InitResponse(String),
    HeartbeatRequest(),
    HeartbeatResponse(u32),
    Json(String),
    Multi(Vec<Package>),
}

pub enum PackageType {
    Unknown,
    InitRequest,
    InitResponse,
    HeartbeatRequest,
    HeartbeatResponse,
    Json,
    Multi,
}

pub mod util {
    pub async fn call_rest_api<Data>(url: String) -> Option<Data>
    where
        Data: serde::de::DeserializeOwned,
    {
        use serde_json::from_str as parse_json;
        use reqwest::{get as http_get, StatusCode};
        use crate::api_schema::rest::RestApiResponse;

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
}

pub mod api_schema {
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
}

pub mod head {
    use binread::BinRead;
    use binwrite::BinWrite;

    pub type HeadBuf = [u8; 16];

    #[derive(BinRead)]
    #[binread(big)]
    #[binread(assert(head_length == 16, "unexpected head length: {}", head_length))]
    #[derive(BinWrite)]
    #[binwrite(big)]
    pub struct Head {
        pub length: u32,
        pub head_length: u16,
        pub proto_ver: u16,
        pub msg_type: u32,
        pub seq: u32,
    }

    #[cfg(test)]
    mod tests {
        #[allow(dead_code)]
        mod examples {
            use hex_literal::hex;
            use super::super::HeadBuf;

            pub const INIT_REQUEST: HeadBuf = hex!("0000 00f9 0010 0001 0000 0007 0000 0001");
            pub const INIT_RESPONSE: HeadBuf = hex!("0000 001a 0010 0001 0000 0008 0000 0001");
            pub const HEARTBEAT_REQUEST: HeadBuf = hex!("0000 001f 0010 0001 0000 0002 0000 0001");
            pub const HEARTBEAT_RESPONSE: HeadBuf = hex!("0000 0014 0010 0001 0000 0003 0000 0000");
            pub const JSON: HeadBuf = hex!("0000 00ff 0010 0000 0000 0005 0000 0000"); // simulated
            pub const MULTI_JSON: HeadBuf = hex!("0000 03d5 0010 0003 0000 0005 0000 0000");
        }

        #[test]
        fn decode_and_encode() {
            use std::io::Cursor;
            use binread::BinReaderExt;
            use binwrite::BinWrite;
            use super::Head;

            let raw = examples::INIT_REQUEST;

            let mut reader = Cursor::new(raw);
            let head: Head = reader.read_ne().unwrap();

            assert_eq!(head.length, 0xf9);
            assert_eq!(head.head_length, 16);
            assert_eq!(head.proto_ver, 1);
            assert_eq!(head.msg_type, 7);
            assert_eq!(head.seq, 1);

            let mut bytes = vec![];
            head.write(&mut bytes).unwrap();
            assert_eq!(raw.to_vec(), bytes);
        }
    }
}

pub mod connect {
    use rand::{seq::SliceRandom, thread_rng as rng};
    use crate::api_schema::rest::HostsInfo;

    pub struct ConnectNeeds {
        pub url: String,
        pub key: String,
    }

    pub fn get_connect_needs(hosts_info: HostsInfo) -> ConnectNeeds {
        let host = &hosts_info.host_list.choose(&mut rng()).unwrap();
        ConnectNeeds {
            url: format!("wss://{}:{}/sub", host.host, host.wss_port),
            key: hosts_info.token,
        }
    }
}
