use foundations::{byterepr::*, byterepr_struct, error_enum};

byterepr_struct! {
    #[derive(Debug)]
    pub struct Head {
        pub length: u32,
        pub head_length: u16,
        pub proto_ver: u16,
        pub msg_type: u32,
        pub seq: u32,
    }
}

impl Head {
    pub const SIZE_16: u16 = Self::SIZE as u16;
    pub const SIZE_32: u32 = Self::SIZE as u32;
    
    pub fn new(msg_type: u32, payload_length: u32) -> Self {
        Self {
            length: Head::SIZE_32 + payload_length,
            head_length: Head::SIZE_16,
            proto_ver: 1,
            msg_type,
            seq: 1,
        }
    }
}

#[derive(PartialEq, Eq, Debug)]
pub enum Package {
    InitRequest(String),
    InitResponse(String),
    HeartbeatRequest,
    HeartbeatResponse(u32),
    Json(String),
    Multi(Vec<Package>),
}

impl Package {
    pub fn decode(raw: &[u8]) -> PackageCodecResult<Package> {
        let (head, payload) = raw.split_at(Head::SIZE);
        let head = Head::from_bytes(head.try_into().unwrap());
        Package::decode_payload(head, payload)
    }

    pub fn decode_payload(head: Head, payload: &[u8]) -> PackageCodecResult<Package> {
        if head.head_length != Head::SIZE_16 {
            return Err(PackageCodecError::UnknownHeadLength(head.head_length));
        }

        let payload_length_head = head.length - Head::SIZE_32;
        let payload_length_acc: u32 = payload.len().try_into()?;
        if payload_length_head != payload_length_acc {
            return Err(PackageCodecError::IncorrectPayloadLength { head: payload_length_head, acc: payload_length_acc });
        }

        // region: macros

        macro_rules! unknown_type {
            () => {
                return Err(PackageCodecError::UnknownPayloadType(head))
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
                2 => Package::HeartbeatRequest,
                7 => Package::InitRequest(string!()),
                _ => unknown_type!(),
            },
            // 2 => Package::unpack(inflate!())?,
            _ => unknown_type!(),
        })
    }

    pub fn encode(self) -> PackageCodecResult<Vec<u8>> {
        Ok(match self {
            Package::HeartbeatRequest => Head::new(2, 0).to_bytes().to_vec(),
            Package::InitRequest(payload) => {
                [
                    &Head::new(7, payload.len().try_into()?).to_bytes(),
                    payload.as_bytes(),
                ].concat()
            },
            _ => return Err(PackageCodecError::NotEncodable),
        })
    }

    fn unpack<B: AsRef<[u8]>>(pack: B) -> PackageCodecResult<Package> {
        let pack = pack.as_ref();
        let total_length = pack.len();
        let mut unpacked = Vec::new();
        let mut offset = 0;
        while offset < total_length {
            let length_buf = pack[offset..offset + 4].try_into()?;
            let length: usize = u32::from_be_bytes(length_buf).try_into()?;
            unpacked.push(Package::decode(&pack[offset..offset + length])?);
            offset += length;
        }
        if offset != total_length {
            return Err(PackageCodecError::UnpackLeak { offset, total_length });
        }
        Ok(Package::Multi(unpacked))
    }

    pub fn flatten(self) -> Vec<Package> {
        let mut flattened = Vec::new();
        fn inner(package: Package, flattened: &mut Vec<Package>) {
            if let Package::Multi(packages) = package {
                for sub_package in packages {
                    inner(sub_package, flattened)
                }
            } else {
                flattened.push(package);
            }
        }
        inner(self, &mut flattened);
        flattened
    }
}

// region: to_json

use serde::Serialize;
use serde_json::{Value as JsonValue, Result as JsonResult};

#[derive(Debug, Serialize)]
#[serde(tag = "type", content = "payload")]
pub enum JsonPackage {
    InitRequest(JsonValue),
    InitResponse(JsonValue),
    HeartbeatRequest,
    HeartbeatResponse(u32),
    Json(JsonValue),
    Multi(Vec<JsonPackage>),
}

impl Package {
    pub fn to_json(&self) -> JsonResult<JsonPackage> {
        Ok(match self {
            Package::InitRequest(s) => JsonPackage::InitRequest(serde_json::from_str(s)?),
            Package::InitResponse(s) => JsonPackage::InitResponse(serde_json::from_str(s)?),
            Package::HeartbeatRequest => JsonPackage::HeartbeatRequest,
            Package::HeartbeatResponse(n) => JsonPackage::HeartbeatResponse(*n),
            Package::Json(s) => JsonPackage::Json(serde_json::from_str(s)?),
            Package::Multi(v) => JsonPackage::Multi(v.iter().map(|p| p.to_json()).collect::<JsonResult<Vec<_>>>()?)
        })
    }
}

// endregion

error_enum! {
    #[derive(Debug)]
    pub enum PackageCodecError {
        UnknownHeadLength(u16),
        IncorrectPayloadLength { head: u32, acc: u32 },
        UnpackLeak { offset: usize, total_length: usize },
        UnknownPayloadType(Head),
        NotEncodable,
    }
    convert {
        IoError          => std::io::Error,
        StringCodecError => std::string::FromUtf8Error,
        BytesSilceError  => std::array::TryFromSliceError,
        SizeConvertError => std::num::TryFromIntError,
    }
}

impl std::fmt::Display for PackageCodecError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for PackageCodecError {}

pub type PackageCodecResult<T> = Result<T, PackageCodecError>;

#[cfg(test)]
mod tests {
    use hex_literal::hex;
    use super::*;

    const _TEST_ROOMID: u32 = 10308958;

    const HEAD_INIT_REQUEST: [u8; Head::SIZE] = hex!("0000 00f9 0010 0001 0000 0007 0000 0001");
    const _HEAD_INIT_RESPONSE: [u8; Head::SIZE] = hex!("0000 001a 0010 0001 0000 0008 0000 0001");
    const _HEAD_HEARTBEAT_REQUEST: [u8; Head::SIZE] = hex!("0000 001f 0010 0001 0000 0002 0000 0001");
    const _HEAD_HEARTBEAT_RESPONSE: [u8; Head::SIZE] = hex!("0000 0014 0010 0001 0000 0003 0000 0000");
    const _HEAD_JSON: [u8; Head::SIZE] = hex!("0000 00ff 0010 0000 0000 0005 0000 0000"); // simulated
    const _HEAD_MULTI_JSON: [u8; Head::SIZE] = hex!("0000 03d5 0010 0003 0000 0005 0000 0000");

    #[test]
    fn head_size() {
        assert_eq!(Head::SIZE, 16);
    }

    #[test]
    fn test_head() {
        let raw = HEAD_INIT_REQUEST;

        let head = Head::from_bytes(raw);
        assert_eq!(raw, head.to_bytes().as_slice());
        assert_eq!(raw, Head::new(7, 0xf9 - Head::SIZE_32).to_bytes().as_slice());

        assert_eq!(head.length, 0xf9);
        assert_eq!(head.head_length, Head::SIZE_16);
        assert_eq!(head.proto_ver, 1);
        assert_eq!(head.msg_type, 7);
        assert_eq!(head.seq, 1);
    }

    const PACKAGE_RAW: [u8; 289] = hex!("000001210010000300000005000000001b7c01002c0e32b9173be1482c4d132ebcf86bd4ac5ab67f8247585ab7e899d9684941a296550487e250572f9cbde7d3855c45cd6486cd6c4213f89e2c3a7de5f4954153694f0f380e4c5a81b52c6901061aca897fb8fdf35f6f58f6900f39a47c9ed5dd6e7fd85dec14ce77532b3e7e3e116e787dfbda3d60de63348668f4ccdcacd4de6825acd4d24c45a5b250ab534449aed4d237c815305c0ff0497069e5dfa4787eb9c46f70e41a353c9213ff5fb6c02de1580d46513449a211981cd6886df9d86f3305f0abb3734e701734600ab59e1b2dd4ac752740a14b1e8a46ab2d794f6ad3b0a058928a4722deffa4f8ca92049a06406052142f61b062f9455bd9203e1604bff1abbb729d30db2520c96e73");
    const PACKAGE_PAYLOAD: &str = "{\"cmd\":\"DANMU_MSG\",\"info\":[[0,1,25,5816798,1631676810606,1631676772,0,\"6420484f\",0,0,0,\"\",0,\"{}\",\"{}\"],\"Hello, LiveKit!!!\",[573732342,\"进栈检票\",1,0,0,10000,1,\"\"],[18,\"滑稽果\",\"老弟一号\",10308958,13081892,\"\",0,13081892,13081892,13081892,0,1,178429408],[13,0,6406234,\"\\u003e50000\",0],[\"\",\"\"],0,0,null,{\"ts\":1631676810,\"ct\":\"2D2BF6C4\"},0,0,null,null,0,91]}";

    macro_rules! pkg_json {
        ($payload:expr) => {
            Package::Json($payload.to_owned())
        };
    }

    #[test]
    fn test_package_decode() {
        assert_eq!(
            Package::decode(&PACKAGE_RAW.to_vec()).unwrap(),
            Package::Multi(vec![pkg_json!(PACKAGE_PAYLOAD)])
        )
    }

    #[test]
    fn flat_related() {
        assert_eq!(
            Package::Multi(vec![
                Package::Multi(vec![
                    Package::Multi(vec![
                        pkg_json!("a"),
                    ]),
                    pkg_json!("b"),
                    pkg_json!("c"),
                ]),
                pkg_json!("d"),
            ])
            .flatten(),
            vec![
                pkg_json!("a"),
                pkg_json!("b"),
                pkg_json!("c"),
                pkg_json!("d"),
            ]
        );
    }

    #[test]
    fn incorrect_heartbeat_response_len() {
        macro_rules! encode {
            ($len:literal, $vec:expr) => {
                [
                    Head::new(3, $len).to_bytes().as_slice(),
                    $vec.as_slice(),
                ].concat()
            };
        }

        let more = encode!(5, [0x00, 0x00, 0x00, 0xff, 0x00]);
        let less = encode!(3, [0x00, 0x00, 0xff]);
        let non_align = encode!(3, [0x00, 0x00, 0x00, 0xff]);

        assert!(matches!(Package::decode(&more), Err(PackageCodecError::BytesSilceError(_))));
        assert!(matches!(Package::decode(&less), Err(PackageCodecError::BytesSilceError(_))));
        assert!(matches!(Package::decode(&non_align), Err(PackageCodecError::IncorrectPayloadLength { .. })));
    }
}
