use serde::{Serialize, Deserialize, de::DeserializeOwned};
use serde_json::{Value as JsonValue, Result as JsonResult};
use crate::package::Package;

// region: (util)

// same as `serde_json::from_value`, but takes reference
#[inline]
pub fn to<D: DeserializeOwned>(value: &JsonValue) -> JsonResult<D>
{
    D::deserialize(value)
}

pub fn numbool(value: &JsonValue) -> JsonResult<bool> {
    let num: u8 = to(value)?;
    if num == 0 {
        Ok(false)
    } else if num == 1 {
        Ok(true)
    } else {
        panic!()
    }
}

/*

pub fn inline_json<D: DeserializeOwned>(value: &JsonValue) -> JsonResult<D>
{
    let json: String = to(value)?;
    Ok(serde_json::from_str(json.as_str())?)
}

pub fn inline_json_opt<D: DeserializeOwned>(value: &JsonValue) -> JsonResult<Option<D>>
{
    let json: String = to(value)?;
    if json == "{}" {
        Ok(None)
    } else {
        Ok(Some(serde_json::from_str(json.as_str())?))
    }
}

*/

pub fn may_inline_json_opt<D: DeserializeOwned>(value: &JsonValue) -> JsonResult<Option<D>>
{
    match value.as_str() {
        None => Ok(Some(to(value)?)),
        Some("{}") => Ok(None),
        Some(json) => Ok(Some(serde_json::from_str(json)?))
    }
}

pub fn string_opt(value: &JsonValue) -> JsonResult<Option<String>> {
    let string: String = to(value)?;
    if string.is_empty() {
        Ok(None)
    } else {
        Ok(Some(string))
    }
}

// todo num_opt
pub fn u32_opt(value: &JsonValue) -> JsonResult<Option<u32>> {
    let num: u32 = to(value)?;
    if num == 0 {
        Ok(None)
    } else {
        Ok(Some(num))
    }
}

pub fn string_u32(value: &JsonValue) -> JsonResult<u32> {
    let string: String = to(value)?;
    Ok(string.parse::<u32>().unwrap())
}

pub fn string_color_to_u32(value: &JsonValue) -> JsonResult<u32> {
    if value.is_string() {
        let string: String = to(value)?;
        let string = {
            assert_eq!(string.len(), 7);
            let mut c = string.chars();
            assert_eq!(c.next(), Some('#'));
            format!("00{}", c.as_str())
        };
        let mut buf = [0u8; 4];
        hex::decode_to_slice(string, &mut buf).unwrap();
        Ok(u32::from_be_bytes(buf))
    } else {
        Ok(to(value)?)
    }
}

// endregion

// region: (common)

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
    pub uid: u64,
    pub uname: String,
    pub live_user_level: u8,
    pub admin: bool,
    pub laoye_monthly: bool,
    pub laoye_annual: bool,
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

// endregion

// region: InitResponse

#[derive(Debug, Deserialize)]
pub struct InitResponse {
    pub code: i32,
}

impl InitResponse {
    pub fn parse<S: AsRef<str>>(raw: S) -> JsonResult<Event> {
        Ok(Event::InitResponse(serde_json::from_str::<InitResponse>(raw.as_ref())?.code))
    }
}

// endregion

// region: Danmaku

#[derive(Debug, Serialize)]
pub struct Danmaku {
    info: DanmakuInfo,
    user: User,
    medal: Option<Medal>,
    emoji: Option<DanmakuEmoji>,
    title: Option<Title>,
}

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

impl Danmaku {
    fn from(raw: &JsonValue) -> JsonResult<Self> {
        let info = &raw[0];
        let user = &raw[2];

        Ok(Danmaku {
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
        })
    }
}

// endregion

// region: Interact

#[derive(Debug, Serialize)]
pub struct Interact {
    kind: InteractKind,
    time: i64, // sec
    uid: u64,
    uname: String,
    medal: Option<Medal>,
}

#[derive(Debug, Serialize)]
pub enum InteractKind {
    Enter,
    Follow,
    Share,
    SpecialFollow,
}

impl InteractKind {
    fn from(value: &JsonValue) -> JsonResult<InteractKind> {
        let num: u32 = to(value)?;
        Ok(match num {
            1 => InteractKind::Enter,
            2 => InteractKind::Follow,
            3 => InteractKind::Share,
            4 => InteractKind::SpecialFollow,
            _ => unreachable!(),
        })
    }
}

impl Interact {
    fn from(raw: &JsonValue) -> JsonResult<Self> {
        Ok(Interact {
            kind: InteractKind::from(&raw["msg_type"])?,
            time: to(&raw["timestamp"])?,
            uid: to(&raw["uid"])?,
            uname: to(&raw["uname"])?,
            medal: Medal::from_common(&raw["fans_medal"])?,
        })
    }
}

// endregion

// region: Gift

#[derive(Debug, Serialize)]
pub struct Gift {
    time: i64, // sec
    uid: u64,
    uname: String,
    uface: String,
    id: i32,
    name: String,
    count: u32,
    medal: Option<Medal>,
}

impl Gift {
    fn from(raw: &JsonValue) -> JsonResult<Self> {
        Ok(Gift {
            time: to(&raw["timestamp"])?,
            uid: to(&raw["uid"])?,
            uname: to(&raw["uname"])?,
            uface: to(&raw["face"])?,
            id: to(&raw["giftId"])?,
            name: to(&raw["giftName"])?,
            count: to(&raw["num"])?,
            medal: Medal::from_common(&raw["medal_info"])?
        })
    }
}

// endregion

// region: GuardBuy

#[derive(Debug, Serialize)]
pub struct GuardBuy {
    time: i64, // sec
    uid: u64,
    uname: String,
    count: u32,
    guard_level: u8,
    price: u32,
}

impl GuardBuy {
    fn from(raw: &JsonValue) -> JsonResult<Self> {
        Ok(GuardBuy {
            time: to(&raw["start_time"])?,
            uid: to(&raw["uid"])?,
            uname: to(&raw["username"])?,
            count: to(&raw["num"])?,
            guard_level: to(&raw["guard_level"])?,
            price: to(&raw["price"])?,
        })
    }
}

// endregion

// region: SuperChat

#[derive(Debug, Serialize)]
pub struct SuperChat {
    time: i64, // sec
    text: String,
    price: u32,
    duration: u32,
    user: User,
    uface: String,
}

impl SuperChat {
    fn from(raw: &JsonValue, user: &JsonValue) -> JsonResult<Self> {
        Ok(SuperChat {
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
        })
    }
}

// endregion

// region: Views

#[derive(Debug, Serialize, Deserialize)]
pub struct Views {
    pub enabled: bool,
    pub views: u32,
}

impl Views {
    fn from(raw: &JsonValue) -> JsonResult<Self> {
        Ok(Views {
            // TODO
            enabled: true,
            views: to(&raw["num"])?,
        })
    }
}

// endregion

// region: RoomStat

#[derive(Debug, Serialize, Deserialize)]
pub struct RoomStat {
    pub fans: u32,
    pub fans_club: u32,
    // red_notice: -1
}

// endregion

// region: RoomInfoChange

#[derive(Debug, Serialize, Deserialize)]
pub struct RoomInfoDiff {
    pub parent_area_name: String,
    pub area_name: String,
    pub title: String,
    pub area_id: u16,
    pub parent_area_id: u8,
}

// endregion

#[derive(Debug, Serialize)]
#[serde(tag = "type", content = "data")]
pub enum Event {
    Popularity(u32),
    InitResponse(i32),

    Danmaku(Danmaku),
    Interact(Interact),
    Gift(Gift),
    GuardBuy(GuardBuy),
    SuperChat(SuperChat),
    Views(Views),

    RoomStat(RoomStat),
    RoomInfoChange(RoomInfoDiff),

    LiveStart,
    LiveEnd,

    Unimplemented { raw: JsonValue },
    Ignored { raw: JsonValue },
    Unknown { raw: JsonValue },

    ParseError { raw: String, error: String },
    CodecError { raw: String, error: String },
}

impl Event {
    pub fn parse<S: AsRef<str>>(raw: S) -> JsonResult<Event> {
        let _raw = raw.as_ref();

        let raw: JsonValue = serde_json::from_str(_raw)?;
        let command: String = to(&raw["cmd"])?;

        Ok(match command.as_str() {
            "DANMU_MSG" => Event::Danmaku(Danmaku::from(&raw["info"])?),
            "INTERACT_WORD" => Event::Interact(Interact::from(&raw["data"])?),
            "SEND_GIFT" => Event::Gift(Gift::from(&raw["data"])?),
            "GUARD_BUY" => Event::GuardBuy(GuardBuy::from(&raw["data"])?),
            "SUPER_CHAT_MESSAGE" => Event::SuperChat(SuperChat::from(&raw["data"], &raw["user_info"])?),
            "WATCHED_CHANGE" => Event::Views(Views::from(&raw["data"])?),

            "ROOM_REAL_TIME_MESSAGE_UPDATE" => Event::RoomStat(to(&raw["data"])?),
            "ROOM_CHANGE" => Event::RoomInfoChange(to(&raw["data"])?),

            "LIVE" => Event::LiveStart,
            "PREPARING" => Event::LiveEnd,

            "ROOM_BLOCK_MSG"
            | "SUPER_CHAT_MESSAGE_DELETE"
            | "LIVE_INTERACTIVE_GAME"
            | "COMBO_SEND"
            | "ENTRY_EFFECT"
            | "SUPER_CHAT_MESSAGE_JPN"
            | "USER_TOAST_MSG"
            | "HOT_ROOM_NOTIFY"
            | "SPECIAL_GIFT"
            | "VOICE_JOIN_ROOM_COUNT_INFO"
            | "VOICE_JOIN_LIST"
            | "VOICE_JOIN_STATUS"
            | "ANCHOR_LOT_CHECKSTATUS"
            | "ANCHOR_LOT_START"
            | "ANCHOR_LOT_END"
            | "ANCHOR_LOT_AWARD" => Event::Unimplemented { raw },

            "STOP_LIVE_ROOM_LIST"
            | "HOT_RANK_CHANGED"
            | "HOT_RANK_CHANGED_V2"
            | "WIDGET_BANNER"
            | "ONLINE_RANK_COUNT"
            | "ONLINE_RANK_V2"
            | "NOTICE_MSG"
            | "ONLINE_RANK_TOP3"
            | "HOT_RANK_SETTLEMENT"
            | "HOT_RANK_SETTLEMENT_V2" => Event::Ignored { raw },

            _ => Event::Unknown { raw },
        })
    }

        fn from_pacakge(package: &Package) -> JsonResult<Event> {
        Ok(match package {
            Package::Json(payload) => Event::parse(payload)?,
            Package::HeartbeatResponse(payload) => Event::Popularity(*payload),
            Package::InitResponse(payload) => InitResponse::parse(payload)?,
            _ => panic!("ImpossibleInFlattened"),
        })
    }

        pub fn from_raw<B: AsRef<[u8]>>(raw: B) -> Vec<Event> {
        let raw = raw.as_ref();
        let mut events = Vec::new();

        match Package::decode(raw) {
            Ok(package) => {
                for flattened in package.flatten() {
                    events.push(match Event::from_pacakge(&flattened) {
                        Ok(event) => event,
                        Err(err) => Event::ParseError {
                            raw: format!("{:?}", &flattened),
                            error: format!("{:?}", err),
                        }
                    })
                }
            }
            Err(err) => {
                events.push(Event::CodecError {
                    raw: hex::encode(raw),
                    error: format!("{:?}", err),
                })
            }
        }

        events
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;
    use super::*;

    #[test]
    fn unknown_cmd() {
        const RAW: &str = "{\"cmd\":\"RUST_YYDS\"}";
        match Event::parse(RAW).unwrap() {
            Event::Unknown { raw } => assert_eq!(raw, RAW),
            _ => unreachable!(),
        }
    }

    #[test]
    fn test_string_color_to_u32() {
        assert_eq!(string_color_to_u32(&json!(42)).unwrap(), 42);
        assert_eq!(string_color_to_u32(&json!("#424242")).unwrap(), 4342338);
    }
}
