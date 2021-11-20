use serde::{Serialize, Deserialize};
use serde_json::{Value as JsonValue, Result as JsonResult};
use crate::util::json::*;

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
    pub medal: Option<Medal>,
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
pub struct Medal {
    pub on: bool,
    pub level: u8,
    pub name: String,
    pub guard: u8,
    pub t_roomid: u32,
    pub t_uid: Option<u32>,
    pub t_uname: Option<String>,
    pub color: u32,
    pub color_border: u32,
    pub color_start: u32,
    pub color_end: u32,
    // pub score: u32, (interact)
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

#[derive(Debug, Serialize)]
pub enum InteractKind {
    Enter,
    Follow,
    Share,
}

impl InteractKind {
    fn new(value: &JsonValue) -> JsonResult<InteractKind> {
        let num: u32 = to(value)?;
        Ok(match num {
            1 => InteractKind::Enter,
            2 => InteractKind::Follow,
            3 => InteractKind::Share,
            _ => unreachable!(),
        })
    }
}

#[derive(Debug, Serialize)]
pub struct Interact {
    pub kind: InteractKind,
    pub time: i64, // sec
    pub uid: u32,
    pub uname: String,
    pub medal: Medal,
}

#[derive(Debug, Serialize)]
pub struct Gift {
    pub medal: Medal,
}

#[derive(Debug, Serialize)]
pub struct RoomInfoChange {
    pub parent_area_name: String,
    pub area_name: String,
    pub title: String,
    pub area_id: u16,
    pub parent_area_id: u8,
}

#[derive(Debug, Serialize)]
pub struct LiveStart {}

#[derive(Debug, Serialize)]
pub struct LiveEnd {}

impl Medal {
    fn new_danmaku(raw: &JsonValue) -> JsonResult<Option<Self>> {
        let medal: Vec<JsonValue> = to(&raw)?;
        if medal.len() == 0 {
            Ok(None)
        } else {
            Ok(Some(Medal {
                on: numbool(&medal[11])?,
                level: to(&medal[0])?,
                name: to(&medal[1])?,
                guard: to(&medal[10])?,
                t_roomid: to(&medal[3])?,
                t_uid: Some(to(&medal[12])?),
                t_uname: Some(to(&medal[2])?),
                color: to(&medal[4])?,
                color_border: to(&medal[7])?,
                color_start: to(&medal[8])?,
                color_end: to(&medal[9])?,
            }))
        }
    }

    fn new_common(medal: &JsonValue) -> JsonResult<Self> {
        Ok(Medal {
            on: numbool(&medal["is_lighted"])?,
            level: to(&medal["medal_level"])?,
            name: to(&medal["medal_name"])?,
            guard: to(&medal["guard_level"])?,
            t_roomid: to(&medal["anchor_roomid"])?,
            t_uid: u32_opt(&medal["target_id"])?,
            t_uname: None,
            color: to(&medal["medal_color"])?,
            color_border: to(&medal["medal_color_border"])?,
            color_start: to(&medal["medal_color_start"])?,
            color_end: to(&medal["medal_color_end"])?,
        })
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
            medal: Medal::new_danmaku(&raw[3])?,
            emoji: inline_json_opt(&info[13])?,
            title: DanmakuTitle(to(&title[0])?, to(&title[1])?),
        })
    }
}

impl Interact {
    pub fn new(raw: &JsonValue) -> JsonResult<Self> {
        Ok(Interact {
            kind: InteractKind::new(&raw["msg_type"])?,
            time: to(&raw["timestamp"])?,
            uid: to(&raw["uid"])?,
            uname: to(&raw["uname"])?,
            medal: Medal::new_common(&raw["fans_medal"])?,
        })
    }
}

impl Gift {
    pub fn new(raw: &JsonValue) -> JsonResult<Self> {
        Ok(Gift {
            medal: Medal::new_common(&raw["medal_info"])?
        })
    }
}
