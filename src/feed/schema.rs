use serde::{Serialize, Deserialize};

#[derive(Serialize)]
pub struct InitRequest {
    pub uid: u32,
    pub roomid: u32,
    pub protover: u8, // unknown number
    pub platform: String,
    pub r#type: u8, // unknown number
    pub key: String,
}

#[derive(Deserialize)]
pub struct InitResponse {
    pub code: i32,
}

// UnknownCommonNumber
type UCN = i32;
// UnknownEnumNumber
type UEN = u8;
// UnknownJsonString
type UJS = String;

pub struct Danmaku (
    DanmakuInfo,
    /// [text] danmaku text
    String,
    DanmakuUser,
    DanmakuMedal,
    DanmakuUserLevel,
);

pub struct DanmakuInfo (
    /// 0
    UCN,
    /// 1[mode]
    UEN,
    /// 2[size]
    UCN,
    /// 3[color] (0-16777215)
    UCN,
    /// 4 (ts-ms)
    u64,
    /// 5[dmid] (ts-s)
    u32,
    /// 6
    UCN,
    /// 7
    String,
    /// 8
    UCN,
    /// 9[type]
    UCN,
    /// 10
    UCN,
    /// 11
    String,
    /// 12 special danmaku type (Text | Emoji | Voice)
    UCN,
    /// 13[emoticonOptions]
    UJS,
    /// 14[voiceConfig]
    UJS,
    // /// 15[modeInfo]
    // ()
);

pub struct DanmakuUser (
    /// [uid] sender uid
    u32,
    /// [nickname] sender uname
    String,
    UCN,
    UCN,
    UCN,
    /// [rank]
    UCN,
    UCN,
    /// ?[uname_color]
    String,
);

pub struct DanmakuMedal (
    /// medal level
    u8,
    /// medal name
    String,
    /// medal owner name
    String,
    /// medal owner roomid
    u32,
    UCN,
    String,
    UCN,
    UCN,
    UCN,
    UCN,
    UCN,
    UCN,
    /// medal owner uid
    u32,
);

pub struct DanmakuUserLevel (
    /// current level
    u8,
    u8,
    u32,
    String,
);
