use serde::{Serialize, Deserialize};
use serde_json::{Value as JsonValue, Result as JsonResult};
use crate::util::json::to;

#[derive(Serialize)]
pub struct InitRequest {
    pub uid: u32,
    pub roomid: u32,
    pub protover: u8,
    pub platform: String,
    pub r#type: u8,
    pub key: String,
}

#[derive(Deserialize)]
pub struct InitResponse {
    pub code: i32,
}

#[derive(Serialize)]
pub struct Danmaku {
    pub time: i64,
    pub color: u32,
    pub text: u32,
    pub uid: u32,
    pub uname: String,
    pub ul: u8,
    pub medal_level: u8,
    pub medal_name: String,
    pub medal_uid: u32,
    pub medal_roomid: u32,
    pub medal_uname: String,
}

impl Danmaku {
    pub fn new(raw: &JsonValue) -> JsonResult<Self> {
        let info = &raw[0];
        let user = &raw[2];
        let medal = &raw[3];
        let ul = &raw[4];

        Ok(Danmaku {
            time: to(&info[5])?,
            color: to(&info[3])?,
            text: to(&raw[1])?,
            uid: to(&user[0])?,
            uname: to(&user[1])?,
            ul: to(&ul[0])?,
            medal_level: to(&medal[0])?,
            medal_name: to(&medal[1])?,
            medal_uid: to(&medal[2])?,
            medal_roomid: to(&medal[3])?,
            medal_uname: to(&medal[12])?,
        })
    }
}
