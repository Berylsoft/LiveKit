use std::convert::TryInto;
use binrw::{BinRead, BinWrite};
use bytes_codec::{BytesDecodeExt, BytesEncodeExt};

#[derive(Debug, serde::Serialize)]
pub struct InitRequest {
    pub uid: u32,
    pub roomid: u32,
    pub protover: u8,
    pub platform: String,
    pub r#type: u8,
    pub key: String,
}

pub const HEAD_LENGTH: u16 = 16;
pub const HEAD_LENGTH_32: u32 = 16;
pub const HEAD_LENGTH_SIZE: usize = 16;
pub type HeadBuf = [u8; HEAD_LENGTH_SIZE];

#[derive(Debug, BinRead, BinWrite)]
#[br(assert(head_length == HEAD_LENGTH, "unexpected head length: {}", head_length))]
pub struct Head {
    pub length: u32,
    pub head_length: u16,
    pub proto_ver: u16,
    pub msg_type: u32,
    pub seq: u32,
}

impl Head {
    pub fn new(msg_type: u32, payload_length: u32) -> Self {
        Self {
            length: HEAD_LENGTH_32 + payload_length,
            head_length: HEAD_LENGTH,
            proto_ver: 1,
            msg_type,
            seq: 1,
        }
    }
}

#[derive(Debug)]
pub enum Package {
    CodecError(Vec<u8>, PackageCodecError),
    InitRequest(String),
    InitResponse(String),
    HeartbeatRequest(),
    HeartbeatResponse(u32),
    Json(String),
    Multi(Vec<Package>),
}

#[derive(Debug)]
pub enum FlatPackage {
    CodecError(Vec<u8>, PackageCodecError),
    InitResponse(String),
    HeartbeatResponse(u32),
    Json(String),
}

impl Package {
    pub fn decode<T: AsRef<[u8]>>(raw: T) -> Self {
        let raw = raw.as_ref();
        match Package::try_decode(raw) {
            Ok(package) => package,
            Err(error) => Package::CodecError(raw.to_vec(), error)
        }
    }

    pub fn try_decode(raw: &[u8]) -> Result<Self, PackageCodecError> {
        let (head, payload) = raw.split_at(HEAD_LENGTH_SIZE);
        let head = Head::decode(head)?;

        // region

        macro_rules! unknown_type {
            () => {
                return Err(PackageCodecError::UnknownType(head))
            };
        }

        macro_rules! string {
            () => {
                String::from_utf8(payload.to_owned())?
            };
        }

        macro_rules! u32 {
            () => {
                u32::from_be_bytes(payload.try_into()?)
            };
        }

        macro_rules! br {
            () => {{
                let mut decoded = Vec::new();
                brotli_decompressor::BrotliDecompress(&mut std::io::Cursor::new(payload), &mut std::io::Cursor::new(&mut decoded))?;
                decoded
            }};
        }

        // endregion

        Ok(match head.proto_ver {
            0 => Package::Json(string!()),
            3 => Package::unpack(br!())?,
            1 => match head.msg_type {
                3 => Package::HeartbeatResponse(u32!()),
                8 => Package::InitResponse(string!()),
                2 => Package::HeartbeatRequest(),
                7 => Package::InitRequest(string!()),
                _ => unknown_type!(),
            },
            // 2 => Package::unpack(inflate!())?,
            _ => unknown_type!(),
        })
    }

    pub fn encode(self) -> Result<Vec<u8>, PackageCodecError> {
        Ok(match self {
            Package::HeartbeatRequest() => Head::new(2, 0).encode()?,
            Package::InitRequest(payload) => {
                let payload = payload.into_bytes();
                [
                    Head::new(7, payload.len().try_into()?).encode()?,
                    payload,
                ].concat()
            },
            _ => return Err(PackageCodecError::NotEncodable),
        })
    }

    pub fn create_init_request(roomid: u32, platform: String, key: String) -> Self {
        Package::InitRequest(
            serde_json::to_string(
                &InitRequest {
                    uid: 0,
                    roomid,
                    protover: 3,
                    platform,
                    r#type: 2,
                    key,
                }
            ).unwrap()
        )
    }

    fn unpack<T: AsRef<[u8]>>(pack: T) -> Result<Self, PackageCodecError> {
        let pack = pack.as_ref();
        let pack_length = pack.len();
        let mut unpacked = Vec::new();
        let mut offset = 0;
        while offset < pack_length {
            let length_buf = pack[offset..offset + 4].try_into()?;
            let length: usize = u32::from_be_bytes(length_buf).try_into()?;
            unpacked.push(Package::try_decode(&pack[offset..offset + length])?);
            offset += length;
        }
        Ok(Package::Multi(unpacked))
    }

    // TODO improve
    pub fn flatten(self) -> Vec<FlatPackage> {
        match self {
            Package::Json(a) => vec![FlatPackage::Json(a)],
            Package::Multi(packages) => packages.into_iter().map(|package| {
                match package {
                    Package::Json(a) => FlatPackage::Json(a),
                    _ => unreachable!(),
                }
            }).collect(),
            Package::HeartbeatResponse(a) => vec![FlatPackage::HeartbeatResponse(a)],
            Package::InitResponse(a) => vec![FlatPackage::InitResponse(a)],
            Package::CodecError(a, b) => vec![FlatPackage::CodecError(a, b)],
            Package::InitRequest(_) | Package::HeartbeatRequest() => unreachable!(),
        }
    }
}

impl FlatPackage {
    pub fn to_json(self) -> Result<serde_json::Value, PackageCodecError> {
        Ok(match self {
            FlatPackage::Json(payload) => serde_json::from_str(payload.as_str())?,
            FlatPackage::HeartbeatResponse(num) => serde_json::json!(num),
            FlatPackage::InitResponse(payload) => serde_json::from_str(payload.as_str())?,
            FlatPackage::CodecError(_, error) => return Err(error),
        })
    }
}

#[derive(Debug)]
pub enum PackageCodecError {
    IoError(std::io::Error),
    StringCodecError(std::string::FromUtf8Error),
    BytesSilceError(std::array::TryFromSliceError),
    NumberConvertError(std::num::TryFromIntError),
    BytesCodecError(binrw::Error),
    UnknownType(Head),
    NotEncodable,
    ToJsonError(serde_json::Error),
}

impl From<std::io::Error> for PackageCodecError {
    fn from(err: std::io::Error) -> PackageCodecError {
        PackageCodecError::IoError(err)
    }
}

impl From<std::string::FromUtf8Error> for PackageCodecError {
    fn from(err: std::string::FromUtf8Error) -> PackageCodecError {
        PackageCodecError::StringCodecError(err)
    }
}

impl From<std::num::TryFromIntError> for PackageCodecError {
    fn from(err: std::num::TryFromIntError) -> PackageCodecError {
        PackageCodecError::NumberConvertError(err)
    }
}

impl From<std::array::TryFromSliceError> for PackageCodecError {
    fn from(err: std::array::TryFromSliceError) -> PackageCodecError {
        PackageCodecError::BytesSilceError(err)
    }
}

impl From<binrw::Error> for PackageCodecError {
    fn from(err: binrw::Error) -> PackageCodecError {
        PackageCodecError::BytesCodecError(err)
    }
}

impl From<serde_json::Error> for PackageCodecError {
    fn from(err: serde_json::Error) -> PackageCodecError {
        PackageCodecError::ToJsonError(err)
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
        assert_eq!(raw.to_vec(), head.encode().unwrap());
        assert_eq!(raw.to_vec(), Head::new(7, 0xf9 - HEAD_LENGTH_32).encode().unwrap());

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
        match package {
            Package::Multi(unpacked) => match &unpacked[0] {
                Package::Json(payload) => assert_eq!(payload, PACKAGE_PAYLOAD),
                _ => panic!(),
            },
            _ => panic!(),
        }
    }

    #[test]
    fn test_init_request() {
        let init = Package::create_init_request(TEST_ROOMID, "web".to_owned(), "key".to_owned());
        match &init {
            Package::InitRequest(payload) => assert!(payload.starts_with(PACKAGE_INIT_BEGINNING)),
            _ => panic!(),
        }
        let init = init.encode().unwrap();
        assert_eq!(init.len(), 94);
    }
}