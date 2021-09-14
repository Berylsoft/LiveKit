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
