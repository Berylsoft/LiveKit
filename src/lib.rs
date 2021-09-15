pub mod rest;
pub mod schema;

pub mod package {
    use std::convert::TryInto;
    use crate::{head::{Head, HEAD_LENGTH_SIZE}, connect::Connect};

    pub enum Package {
        Unknown(Vec<u8>),
        InitRequest(String),
        InitResponse(String),
        HeartbeatRequest(),
        HeartbeatResponse(u32),
        Json(String),
        Multi(Vec<Package>),
    }

    impl Package {
        pub fn decode(raw: Vec<u8>) -> Self {
            let (head, payload) = raw.split_at(HEAD_LENGTH_SIZE);

            match Head::decode(head) {
                Some(head) => match head.msg_type {
                    5 => match head.proto_ver {
                        0 => Package::Json(String::from_utf8(payload.to_vec()).unwrap()),
                        2 => unimplemented!(),
                        3 => unimplemented!(),
                        _ => Package::Unknown(raw),
                    }
                    2 => Package::HeartbeatRequest(),
                    3 => Package::HeartbeatResponse(u32::from_be_bytes(payload[0..4].try_into().unwrap())),
                    7 => Package::InitRequest(String::from_utf8(payload.to_vec()).unwrap()),
                    8 => Package::InitResponse(String::from_utf8(payload.to_vec()).unwrap()),
                    _ => Package::Unknown(raw),
                },
                None => Package::Unknown(raw),
            }
        }

        pub fn encode(self) -> Vec<u8> {
            match self {
                Package::HeartbeatRequest() => Head::new(2, 0).encode(),
                Package::InitRequest(payload) => {
                    let mut payload = payload.into_bytes();
                    let mut buf = Head::new(7, payload.len().try_into().unwrap()).encode();
                    buf.append(&mut payload);
                    buf
                },
                _ => unreachable!(),
            }
        }

        pub fn create_init_request(connect: Connect) -> Self {
            use serde_json::to_string as build_json;
            use crate::schema::ConnectInfo;

            Package::InitRequest(build_json(&ConnectInfo {
                uid: 0,
                roomid: connect.roomid,
                protover: 2,
                platform: "web".to_string(),
                r#type: 2,
                key: connect.key.to_string(),
            }).unwrap())
        }
    }
}

pub mod head {
    use std::io::Cursor;
    use binread::{BinRead, BinReaderExt};
    use binwrite::BinWrite;

    pub const HEAD_LENGTH: u16 = 16;
    pub const HEAD_LENGTH_32: u32 = 16;
    pub const HEAD_LENGTH_SIZE: usize = 16;
    pub type HeadBuf = [u8; HEAD_LENGTH_SIZE];

    #[derive(BinRead)]
    #[binread(big)]
    #[binread(assert(head_length == HEAD_LENGTH, "unexpected head length: {}", head_length))]
    #[derive(BinWrite)]
    #[binwrite(big)]
    pub struct Head {
        pub length: u32,
        pub head_length: u16,
        pub proto_ver: u16,
        pub msg_type: u32,
        pub seq: u32,
    }

    impl Head {
        pub fn decode(raw: &[u8]) -> Option<Self> {
            let mut reader = Cursor::new(raw);
            reader.read_be().ok()?
        }

        pub fn encode(&self) -> Vec<u8> {
            let mut bytes = vec![];
            self.write(&mut bytes).unwrap();
            bytes
        }

        pub fn new(msg_type: u32, payload_length: u32) -> Self {
            Head {
                length: HEAD_LENGTH_32 + payload_length,
                head_length: HEAD_LENGTH,
                proto_ver: 1,
                msg_type,
                seq: 1,
            }
        }
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
        fn test() {
            use super::{Head, HEAD_LENGTH, HEAD_LENGTH_32};

            let raw = examples::INIT_REQUEST;

            let head = Head::decode(&raw).unwrap();
            assert_eq!(raw.to_vec(), head.encode());
            assert_eq!(raw.to_vec(), Head::new(7, 0xf9 - HEAD_LENGTH_32).encode());

            assert_eq!(head.length, 0xf9);
            assert_eq!(head.head_length, HEAD_LENGTH);
            assert_eq!(head.proto_ver, 1);
            assert_eq!(head.msg_type, 7);
            assert_eq!(head.seq, 1);
        }
    }
}

pub mod connect {
    use rand::{seq::SliceRandom, thread_rng as rng};
    use crate::rest::HostsInfo;

    pub struct Connect {
        pub roomid: u32,
        pub url: String,
        pub key: String,
    }
    
    impl Connect {
        pub async fn new(roomid: u32) -> Option<Self> {
            let hosts_info = HostsInfo::get(roomid).await?;
            let host = &hosts_info.host_list.choose(&mut rng()).unwrap();
            Some(Connect {
                roomid: roomid,
                url: format!("wss://{}:{}/sub", host.host, host.wss_port),
                key: hosts_info.token,
            })
        }
    }
}
