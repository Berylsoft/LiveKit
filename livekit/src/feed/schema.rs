use serde::{Serialize, Deserialize};
use serde_json::{Value as JsonValue, Result as JsonResult};
use crate::util::json::{to, numbool, inline_json_opt};

#[derive(Debug, Serialize)]
pub struct InitRequest {
    pub uid: u32,
    pub roomid: u32,
    pub protover: u8,
    pub platform: String,
    pub r#type: u8,
    pub key: String,
}

#[derive(Debug, Serialize)]
pub struct InitResponse {
    pub code: i32,
}

#[derive(Debug, Serialize)]
pub struct Danmaku {
    pub info: DanmakuInfo,
    pub user: DanmakuUser,
    pub medal: Option<DanmakuMedal>,
    pub emoji: Option<DanmakuEmoji>,
    pub title: DanmakuTitle,
}

#[derive(Debug, Serialize)]
pub struct DanmakuInfo {
    pub time: i64,
    pub text: String,
    pub color: u32,
    pub size: u32,
    pub rand: i64,
}

#[derive(Debug, Serialize)]
pub struct DanmakuUser {
    pub uid: u32,
    pub uname: String,
    pub live_user_level: u8,
    pub admin: bool,
    pub laoye_monthly: bool,
    pub laoye_annual: bool,
}

#[derive(Debug, Serialize)]
pub struct DanmakuMedal {
    pub on: bool,
    pub level: u8,
    pub name: String,
    pub guard: u8,
    pub t_uid: u32,
    pub t_roomid: u32,
    pub t_uname: String,
    pub color: u32,
    pub color_border: u32,
    pub color_start: u32,
    pub color_end: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DanmakuEmoji {
    pub height: i32,
    pub in_player_area: i32,
    pub is_dynamic: i32,
    pub url: String,
    pub width: i32,
}

#[derive(Debug, Serialize)]
pub struct DanmakuTitle(String, String);

impl DanmakuMedal {
    fn new(raw: &JsonValue) -> JsonResult<Option<Self>> {
        let medal: Vec<JsonValue> = to(&raw)?;
        if medal.len() == 0 {
            Ok(None)
        } else {
            Ok(Some(DanmakuMedal {
                on: numbool(&medal[11])?,
                level: to(&medal[0])?,
                name: to(&medal[1])?,
                guard: to(&medal[10])?,
                t_uid: to(&medal[12])?,
                t_roomid: to(&medal[3])?,
                t_uname: to(&medal[2])?,
                color: to(&medal[4])?,
                color_border: to(&medal[7])?,
                color_start: to(&medal[8])?,
                color_end: to(&medal[9])?,
            }))
        }
    }
}

impl Danmaku {
    pub fn new(raw: &JsonValue) -> JsonResult<Self> {
        let info = &raw[0];
        let user = &raw[2];
        let title = &raw[5];

        Ok(Danmaku {
            info: DanmakuInfo {
                time: to(&info[4])?,
                text: to(&raw[1])?,
                color: to(&info[3])?,
                size: to(&info[2])?,
                rand: to(&info[5])?,
            },
            user: DanmakuUser {
                uid: to(&user[0])?,
                uname: to(&user[1])?,
                live_user_level: to(&raw[4][0])?,
                admin: numbool(&user[2])?,
                laoye_monthly: numbool(&user[3])?,
                laoye_annual: numbool(&user[4])?,
            },
            medal: DanmakuMedal::new(&raw[3])?,
            emoji: inline_json_opt(&info[13])?,
            title: DanmakuTitle(to(&title[0])?, to(&title[1])?),
        })
    }
}
