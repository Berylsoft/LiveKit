use std::{convert::TryInto, io::Cursor};
use binread::{BinRead, BinReaderExt};
use binwrite::BinWrite;
use crate::{
    util::{compress::{de_brotli, inflate}, vec},
    schema::ConnectInfo,
    client::{Sender, Event},
};

pub const HEAD_LENGTH: u16 = 16;
pub const HEAD_LENGTH_32: u32 = 16;
pub const HEAD_LENGTH_SIZE: usize = 16;
pub type HeadBuf = [u8; HEAD_LENGTH_SIZE];

#[derive(BinRead, BinWrite)]
#[binread(big)]
#[binread(assert(head_length == HEAD_LENGTH, "unexpected head length: {}", head_length))]
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
    pub fn decode(raw: &Vec<u8>) -> Self {
        let (head, payload) = raw.split_at(HEAD_LENGTH_SIZE);
        match Head::decode(head) {
            Some(head) => match head.msg_type {
                5 => match head.proto_ver {
                    0 => Package::Json(String::from_utf8(payload.to_vec()).unwrap()),
                    3 => Package::unpack(de_brotli(payload).unwrap()),
                    2 => Package::unpack(inflate(payload).unwrap()),
                    _ => Package::Unknown(raw.clone()),
                },
                2 => Package::HeartbeatRequest(),
                3 => Package::HeartbeatResponse(u32::from_be_bytes(payload[0..4].try_into().unwrap())),
                7 => Package::InitRequest(String::from_utf8(payload.to_vec()).unwrap()),
                8 => Package::InitResponse(String::from_utf8(payload.to_vec()).unwrap()),
                _ => Package::Unknown(raw.clone()),
            },
            None => Package::Unknown(raw.clone()),
        }
    }

    pub fn encode(self) -> Vec<u8> {
        match self {
            Package::HeartbeatRequest() => Head::new(2, 0).encode(),
            Package::InitRequest(payload) => vec::concat(
                Head::new(7, payload.len().try_into().unwrap()).encode(),
                payload.into_bytes(),
            ),
            _ => unreachable!(),
        }
    }

    pub fn create_init_request(roomid: u32, key: String) -> Self {
        Package::InitRequest(
            serde_json::to_string(
                &ConnectInfo {
                    uid: 0,
                    roomid,
                    protover: 3,
                    platform: "web".to_string(),
                    r#type: 2,
                    key,
                }
            ).unwrap()
        )
    }

    fn unpack(pack: Vec<u8>) -> Self {
        let pack_length = pack.len();
        let mut unpacked = Vec::new();
        let mut offset = 0;
        while offset < pack_length {
            let length_buf = pack[offset..offset + 4].try_into().unwrap();
            let length: usize = u32::from_be_bytes(length_buf).try_into().unwrap();
            let raw = (&pack[offset..offset + length]).to_vec();
            unpacked.push(Package::decode(&raw));
            offset += length;
        }
        Package::Multi(unpacked)
    }

    pub fn send_as_events(self, channel_sender: &mut Sender) {
        // TODO process recursive `Multi` & return iter
        match self {
            Package::Multi(payloads) => for payload in payloads {
                match payload {
                    Package::Json(payload) => { channel_sender.send(Event::Message(payload)).unwrap(); },
                    _ => unreachable!(),
                }
            },
            Package::Json(payload) => { channel_sender.send(Event::Message(payload)).unwrap(); },
            Package::HeartbeatResponse(payload) => { channel_sender.send(Event::Popularity(payload)).unwrap(); },
            Package::InitResponse(_) => (),
            _ => unreachable!(),
        }
    }
}

#[cfg(test)]
mod tests {
    use hex_literal::hex;
    use super::*;

    const TEST_ROOMID: u32 = 10308958;

    const HEAD_INIT_REQUEST: HeadBuf = hex!("0000 00f9 0010 0001 0000 0007 0000 0001");
    const _HEAD_INIT_RESPONSE: HeadBuf = hex!("0000 001a 0010 0001 0000 0008 0000 0001");
    const _HEAD_HEARTBEAT_REQUEST: HeadBuf = hex!("0000 001f 0010 0001 0000 0002 0000 0001");
    const _HEAD_HEARTBEAT_RESPONSE: HeadBuf = hex!("0000 0014 0010 0001 0000 0003 0000 0000");
    const _HEAD_JSON: HeadBuf = hex!("0000 00ff 0010 0000 0000 0005 0000 0000"); // simulated
    const _HEAD_MULTI_JSON: HeadBuf = hex!("0000 03d5 0010 0003 0000 0005 0000 0000");

    #[test]
    fn test_head() {
        let raw = HEAD_INIT_REQUEST;

        let head = Head::decode(&raw).unwrap();
        assert_eq!(raw.to_vec(), head.encode());
        assert_eq!(raw.to_vec(), Head::new(7, 0xf9 - HEAD_LENGTH_32).encode());

        assert_eq!(head.length, 0xf9);
        assert_eq!(head.head_length, HEAD_LENGTH);
        assert_eq!(head.proto_ver, 1);
        assert_eq!(head.msg_type, 7);
        assert_eq!(head.seq, 1);
    }

    const PACKAGE_RAW: [u8; 289] = hex!("000001210010000300000005000000001b7c01002c0e32b9173be1482c4d132ebcf86bd4ac5ab67f8247585ab7e899d9684941a296550487e250572f9cbde7d3855c45cd6486cd6c4213f89e2c3a7de5f4954153694f0f380e4c5a81b52c6901061aca897fb8fdf35f6f58f6900f39a47c9ed5dd6e7fd85dec14ce77532b3e7e3e116e787dfbda3d60de63348668f4ccdcacd4de6825acd4d24c45a5b250ab534449aed4d237c815305c0ff0497069e5dfa4787eb9c46f70e41a353c9213ff5fb6c02de1580d46513449a211981cd6886df9d86f3305f0abb3734e701734600ab59e1b2dd4ac752740a14b1e8a46ab2d794f6ad3b0a058928a4722deffa4f8ca92049a06406052142f61b062f9455bd9203e1604bff1abbb729d30db2520c96e73");
    const PACKAGE_PAYLOAD: &str = "{\"cmd\":\"DANMU_MSG\",\"info\":[[0,1,25,5816798,1631676810606,1631676772,0,\"6420484f\",0,0,0,\"\",0,\"{}\",\"{}\"],\"Hello, LiveKit!!!\",[573732342,\"进栈检票\",1,0,0,10000,1,\"\"],[18,\"滑稽果\",\"老弟一号\",10308958,13081892,\"\",0,13081892,13081892,13081892,0,1,178429408],[13,0,6406234,\"\\u003e50000\",0],[\"\",\"\"],0,0,null,{\"ts\":1631676810,\"ct\":\"2D2BF6C4\"},0,0,null,null,0,91]}";
    const PACKAGE_INIT_BEGINNING: &str = "{\"uid\":0,\"roomid\":10308958,\"protover\":3,\"platform\":\"web\",\"type\":2,\"key\":\"";

    #[test]
    fn test_package_decode() {
        let package = Package::decode(&PACKAGE_RAW.to_vec());
        if let Package::Multi(unpacked) = package {
            if let Package::Json(payload) = &unpacked[0] {
                assert_eq!(payload, PACKAGE_PAYLOAD);
            } else { panic!() }
        } else { panic!() }
    }

    #[tokio::test]
    async fn test_init_request() {
        let init = Package::create_init_request(TEST_ROOMID, "key".to_string());
        if let Package::InitRequest(payload) = &init {
            assert!(payload.starts_with(PACKAGE_INIT_BEGINNING))
        } else { panic!() }
        let init = init.encode();
        assert_eq!(init.len(), 94);
    }
}
