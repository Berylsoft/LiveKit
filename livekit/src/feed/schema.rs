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
pub enum Event {
    Unknown { raw: String },
    ParseError { raw: String, error: String },

    Danmaku {
        info: DanmakuInfo,
        user: DanmakuUser,
        medal: Option<Medal>,
        emoji: Option<DanmakuEmoji>,
        title: DanmakuTitle,
    },

    Interact {
        kind: InteractKind,
        time: i64, // sec
        uid: u32,
        uname: String,
        medal: Option<Medal>,
    },

    Gift {
        time: i64, // sec
        uid: u32,
        uname: String,
        uface: String,
        id: i32,
        name: String,
        count: u32,
        medal: Option<Medal>,
    },

    RoomInfoChange(RoomInfoDiff),

    LiveStart,

    LiveEnd,
}

impl Event {
    fn new_inner(raw: &str) -> JsonResult<Event> {
        let unknown = || Event::Unknown { raw: "raw".to_owned() };

        let raw: JsonValue = serde_json::from_str(raw)?;
        let command: String = to(&raw["cmd"])?;

        Ok(match command.as_str() {
            "DANMU_MSG" => {
                let raw = &raw["info"];

                let info = &raw[0];
                let user = &raw[2];
                let title = &raw[5];

                Event::Danmaku {
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
                    emoji: may_inline_json_opt(&info[13])?,
                    title: DanmakuTitle(to(&title[0])?, to(&title[1])?),
                }
            },

            "INTERACT_WORD" => {
                let raw = &raw["data"];

                Event::Interact {
                    kind: InteractKind::new(&raw["msg_type"])?,
                    time: to(&raw["timestamp"])?,
                    uid: to(&raw["uid"])?,
                    uname: to(&raw["uname"])?,
                    medal: Medal::new_common(&raw["fans_medal"])?,
                }
            },

            "SEND_GIFT" => {
                let raw = &raw["data"];

                Event::Gift {
                    time: to(&raw["timestamp"])?,
                    uid: to(&raw["uid"])?,
                    uname: to(&raw["uname"])?,
                    uface: to(&raw["face"])?,
                    id: to(&raw["giftId"])?,
                    name: to(&raw["giftName"])?,
                    count: to(&raw["num"])?,
                    medal: Medal::new_common(&raw["medal_info"])?
                }
            },

            "ROOM_CHANGE" => {
                Event::RoomInfoChange(to(&raw["data"])?)
            }

            "LIVE" => {
                Event::LiveStart
            }

            "PREPARING" => {
                Event::LiveEnd
            }

            _ => unknown(),
        })
    }

    pub fn new(raw: String) -> Event {
        match Event::new_inner(raw.as_str()) {
            Ok(event) => event,
            Err(err) => {
                Event::ParseError { raw, error: format!("{:?}", err) }
            }
        }
    }
}

// common

#[derive(Debug, Serialize)]
pub struct Medal {
    pub on: bool,
    pub level: u8,
    pub name: String,
    pub guard: u8,
    pub t_roomid: Option<u32>,
    pub t_uid: Option<u32>,
    pub t_uname: Option<String>,
    pub color: u32,
    pub color_border: u32,
    pub color_start: u32,
    pub color_end: u32,
    // pub score: u32, (interact)
}

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

    fn new_common(medal: &JsonValue) -> JsonResult<Option<Self>> {
        let name: String = to(&medal["medal_name"])?;
        if name.len() == 0 {
            Ok(None)
        } else {
            Ok(Some(Medal {
                on: numbool(&medal["is_lighted"])?,
                level: to(&medal["medal_level"])?,
                name: to(&medal["medal_name"])?,
                guard: to(&medal["guard_level"])?,
                t_roomid: u32_opt(&medal["anchor_roomid"])?,
                t_uid: u32_opt(&medal["target_id"])?,
                t_uname: None,
                color: to(&medal["medal_color"])?,
                color_border: to(&medal["medal_color_border"])?,
                color_start: to(&medal["medal_color_start"])?,
                color_end: to(&medal["medal_color_end"])?,
            }))
        }
    }
}

// Danmaku

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

// Interact

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

// RoomInfoChange

#[derive(Debug, Serialize, Deserialize)]
pub struct RoomInfoDiff {
    parent_area_name: String,
    area_name: String,
    title: String,
    area_id: u16,
    parent_area_id: u8,
}

#[cfg(test)]
mod tests {
    use super::Event;

    #[test]
    fn unknown_cmd() {
        let event = Event::new_inner("{\"cmd\":\"RUST_YYDS\"}").unwrap();
        assert!(matches!(event, Event::Unknown { raw: _ }));
    }

    #[test]
    fn test() {
        println!("{:?}", Event::new_inner("{\"cmd\":\"DANMU_MSG\",\"info\":[[0,1,25,5816798,1637435549015,-1476290639,0,\"e9029e7c\",0,0,0,\"\",1,{\"bulge_display\":1,\"emoticon_unique\":\"room_947866_18\",\"height\":162,\"in_player_area\":1,\"is_dynamic\":1,\"url\":\"http://i0.hdslb.com/bfs/live/bbed541df7ed9678945c89ee292b21c515bd5998.png\",\"width\":162},\"{}\",{\"mode\":0,\"extra\":\"{\\\"send_from_me\\\":false,\\\"content\\\":\\\"room_947866_18\\\",\\\"mode\\\":0,\\\"font_size\\\":25,\\\"color\\\":5816798,\\\"user_hash\\\":\\\"3909262972\\\",\\\"direction\\\":0,\\\"pk_direction\\\":0}\"}],\"awsl\",[499392950,\"凛冬之辰\",0,0,0,10000,1,\"\"],[16,\"小呦西\",\"扶桑大红花丶\",947866,12478086,\"\",0,12478086,12478086,12478086,0,1,3985676],[14,0,6406234,\"\\u003e50000\",0],[\"\",\"\"],0,0,null,{\"ts\":1637435549,\"ct\":\"FF72EC20\"},0,0,null,null,0,63]}"));
        panic!();
    }
}
