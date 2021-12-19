use serde::{Serialize, Deserialize};
use serde_json::{Value as JsonValue, Result as JsonResult};
use crate::util::json::*;
#[cfg(feature = "package")]
use crate::package::FlatPackage;

#[derive(Debug, Deserialize)]
pub struct InitResponse {
    pub code: i32,
}

#[derive(Debug, Serialize)]
pub enum Event {
    Unknown { raw: String },
    ParseError { raw: String, error: String },
    CodecError { raw: String, error: String },
    Unimplemented,
    Ignored,

    Popularity(u32),
    InitResponse(i32),

    Danmaku {
        info: DanmakuInfo,
        user: User,
        medal: Option<Medal>,
        emoji: Option<DanmakuEmoji>,
        title: Option<Title>,
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

    GuardBuy {
        time: i64, // sec
        uid: u32,
        uname: String,
        count: u32,
        guard_level: u8,
        price: u32,
    },

    SuperChat {
        time: i64, // sec
        text: String,
        price: u32,
        duration: u32,
        user: User,
        uface: String,
    },

    RoomStat(RoomStat),

    RoomInfoChange(RoomInfoDiff),

    LiveStart,

    LiveEnd,
}

impl Event {
    pub fn parse<Str: AsRef<str>>(raw: Str) -> JsonResult<Event> {
        let raw = raw.as_ref();

        let unknown = || Event::Unknown { raw: raw.to_owned() };

        let raw: JsonValue = serde_json::from_str(raw)?;
        let command: String = to(&raw["cmd"])?;

        Ok(match command.as_str() {
            "DANMU_MSG" => {
                let raw = &raw["info"];

                let info = &raw[0];
                let user = &raw[2];

                Event::Danmaku {
                    info: DanmakuInfo {
                        time: to(&info[4])?,
                        text: to(&raw[1])?,
                        color: to(&info[3])?,
                        size: to(&info[2])?,
                        rand: to(&info[5])?,
                    },
                    user: User {
                        uid: to(&user[0])?,
                        uname: to(&user[1])?,
                        live_user_level: to(&raw[4][0])?,
                        admin: numbool(&user[2])?,
                        laoye_monthly: numbool(&user[3])?,
                        laoye_annual: numbool(&user[4])?,
                    },
                    medal: Medal::from_danmaku(&raw[3])?,
                    emoji: may_inline_json_opt(&info[13])?,
                    title: Title::from(&raw[5])?,
                }
            },

            "INTERACT_WORD" => {
                let raw = &raw["data"];

                Event::Interact {
                    kind: InteractKind::new(&raw["msg_type"])?,
                    time: to(&raw["timestamp"])?,
                    uid: to(&raw["uid"])?,
                    uname: to(&raw["uname"])?,
                    medal: Medal::from_common(&raw["fans_medal"])?,
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
                    medal: Medal::from_common(&raw["medal_info"])?
                }
            },

            "SUPER_CHAT_MESSAGE" => {
                let raw = &raw["data"];
                let user = &raw["user_info"];

                Event::SuperChat {
                    time: to(&raw["ts"])?,
                    text: to(&raw["message"])?,
                    price: to(&raw["price"])?,
                    duration: to(&raw["time"])?,
                    user: User {
                        uid: to(&raw["uid"])?,
                        uname: to(&user["uname"])?,
                        live_user_level: to(&user["user_level"])?,
                        admin: numbool(&user["manager"])?,
                        laoye_monthly: numbool(&user["is_vip"])?,
                        laoye_annual: numbool(&user["is_svip"])?,
                    },
                    uface: to(&user["face"])?,
                }
            },

            "GUARD_BUY" => {
                let raw = &raw["data"];

                Event::GuardBuy {
                    time: to(&raw["start_time"])?,
                    uid: to(&raw["uid"])?,
                    uname: to(&raw["username"])?,
                    count: to(&raw["num"])?,
                    guard_level: to(&raw["guard_level"])?,
                    price: to(&raw["price"])?,
                }
            }

            "ROOM_REAL_TIME_MESSAGE_UPDATE" => {
                Event::RoomStat(to(&raw["data"])?)
            }

            "ROOM_CHANGE" => {
                Event::RoomInfoChange(to(&raw["data"])?)
            },

            "LIVE" => {
                Event::LiveStart
            },

            "PREPARING" => {
                Event::LiveEnd
            },

            "LIVE_INTERACTIVE_GAME" | "COMBO_SEND" | "ENTRY_EFFECT" | "SUPER_CHAT_MESSAGE_JPN" | "USER_TOAST_MSG" | "HOT_ROOM_NOTIFY" | "SPECIAL_GIFT" | "VOICE_JOIN_ROOM_COUNT_INFO" | "VOICE_JOIN_LIST" | "VOICE_JOIN_STATUS" => {
                Event::Unimplemented
            },

            "STOP_LIVE_ROOM_LIST" | "HOT_RANK_CHANGED" | "HOT_RANK_CHANGED_V2" | "WIDGET_BANNER" | "ONLINE_RANK_COUNT" | "ONLINE_RANK_V2" | "NOTICE_MSG" | "ONLINE_RANK_TOP3" | "HOT_RANK_SETTLEMENT" | "HOT_RANK_SETTLEMENT_V2" => {
                Event::Ignored
            },

            _ => unknown(),
        })
    }

    #[cfg(feature = "package")]
    pub fn from_package(package: FlatPackage) -> Event {
        match package {
            FlatPackage::Json(payload) => {
                match Event::parse(payload.as_str()) {
                    Ok(event) => event,
                    Err(err) => Event::ParseError {
                        raw: payload,
                        error: format!("{:?}", err),
                    },
                }
            },
            FlatPackage::HeartbeatResponse(payload) => {
                Event::Popularity(payload)
            },
            FlatPackage::InitResponse(payload) => {
                match serde_json::from_str::<InitResponse>(payload.as_str()) {
                    Ok(payload) => Event::InitResponse(payload.code),
                    Err(err) => Event::ParseError {
                        raw: payload,
                        error: format!("{:?}", err),
                    },
                }
            },
            FlatPackage::CodecError(raw, err) => {
                Event::CodecError {
                    raw: hex::encode(raw),
                    error: format!("{:?}", err),
                }
            },
        }
    }
}

// common

#[derive(Debug, Serialize)]
pub struct Medal {
    pub on: bool,
    pub level: u8,
    pub name: String,
    pub guard_level: u8,
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
    fn from_danmaku(raw: &JsonValue) -> JsonResult<Option<Self>> {
        let medal: Vec<JsonValue> = to(&raw)?;
        if medal.len() == 0 {
            Ok(None)
        } else {
            Ok(Some(Medal {
                on: numbool(&medal[11])?,
                level: to(&medal[0])?,
                name: to(&medal[1])?,
                guard_level: to(&medal[10])?,
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

    fn from_common(medal: &JsonValue) -> JsonResult<Option<Self>> {
        let name: String = to(&medal["medal_name"])?;
        if name.len() == 0 {
            Ok(None)
        } else {
            Ok(Some(Medal {
                on: numbool(&medal["is_lighted"])?,
                level: to(&medal["medal_level"])?,
                name: to(&medal["medal_name"])?,
                guard_level: to(&medal["guard_level"])?,
                t_roomid: u32_opt(&medal["anchor_roomid"])?,
                t_uid: u32_opt(&medal["target_id"])?,
                t_uname: None,
                color: string_color_to_u32(&medal["medal_color"])?,
                color_border: to(&medal["medal_color_border"])?,
                color_start: to(&medal["medal_color_start"])?,
                color_end: to(&medal["medal_color_end"])?,
            }))
        }
    }
}

#[derive(Debug, Serialize)]
pub struct User {
    pub uid: u32,
    pub uname: String,
    pub live_user_level: u8,
    pub admin: bool,
    pub laoye_monthly: bool,
    pub laoye_annual: bool,
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

#[derive(Debug, Serialize, Deserialize)]
pub struct DanmakuEmoji {
    pub height: i32,
    pub in_player_area: i32,
    pub is_dynamic: i32,
    pub url: String,
    pub width: i32,
}

#[derive(Debug, Serialize)]
pub struct Title(String, Option<String>);

impl Title {
    fn from(raw: &JsonValue) -> JsonResult<Option<Self>> {
        Ok(match string_opt(&raw[0])? {
            None => None,
            Some(first) => match string_opt(&raw[1])? {
                None => Some(Title(first, None)),
                Some(second) => {
                    if first == second {
                        Some(Title(first, None))
                    } else {
                        Some(Title(first, Some(second)))
                    }
                },
            },
        })
    }
}

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

// RoomStat

#[derive(Debug, Serialize, Deserialize)]
pub struct RoomStat {
    pub fans: u32,
    pub fans_club: u32,
    // red_notice: -1
}

// RoomInfoChange

#[derive(Debug, Serialize, Deserialize)]
pub struct RoomInfoDiff {
    pub parent_area_name: String,
    pub area_name: String,
    pub title: String,
    pub area_id: u16,
    pub parent_area_id: u8,
}

#[cfg(test)]
mod tests {
    use super::Event;

    #[test]
    fn unknown_cmd() {
        let event = Event::parse("{\"cmd\":\"RUST_YYDS\"}").unwrap();
        assert!(matches!(event, Event::Unknown { raw: _ }));
    }
}
