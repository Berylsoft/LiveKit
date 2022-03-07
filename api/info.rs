use serde::{Serialize, Deserialize};
use crate::client::{RestApi, RestApiRequestKind};

#[derive(Serialize)]
pub struct GetRoomInfo {
    pub sroomid: u32,
}

impl RestApi for GetRoomInfo {
    type Response = RoomInfo;

    fn kind(&self) -> RestApiRequestKind {
        RestApiRequestKind::BareGet
    }

    fn path(&self) -> String {
        format!(
            "/room/v1/Room/get_info?id={}",
            self.sroomid,
        )
    }
}

#[derive(Clone, Deserialize)]
pub struct RoomInfo {
    pub uid: u32,
    pub room_id: u32,
    pub short_id: u32,
    pub live_status: u8,
    pub parent_area_name: String,
    pub area_name: String,
    pub title: String,
    pub attention: u32,
    pub online: u32,
    pub is_portrait: bool,
    pub description: String,
    pub area_id: u16,
    pub parent_area_id: u8,
    pub background: String,
    pub user_cover: String,
    pub keyframe: String,
    pub tags: String,
}

#[derive(Clone, Deserialize)]
pub struct UserInfoInfo {
    pub uname: String,
}

#[derive(Clone, Deserialize)]
pub struct UserInfoLevelMaster {
    pub level: u32,
}

#[derive(Clone, Deserialize)]
pub struct UserInfoLevel {
    pub master_level: UserInfoLevelMaster,
}

#[derive(Clone, Deserialize)]
pub struct UserInfo {
    pub info: UserInfoInfo,
    pub level: UserInfoLevel,
}

#[derive(Serialize)]
pub struct GetUserInfo {
    pub sroomid: u32,
}

impl RestApi for GetUserInfo {
    type Response = UserInfo;

    fn kind(&self) -> RestApiRequestKind {
        RestApiRequestKind::BareGet
    }

    fn path(&self) -> String {
        format!(
            "/live_user/v1/UserInfo/get_anchor_in_room?roomid={}",
            self.sroomid,
        )
    }
}

#[derive(Serialize)]
pub struct GetExtRoomInfo {
    pub sroomid: u32,
}

impl RestApi for GetExtRoomInfo {
    type Response = ExtRoomInfo;

    fn kind(&self) -> RestApiRequestKind {
        RestApiRequestKind::BareGet
    }

    fn path(&self) -> String {
        format!(
            "/xlive/web-room/v1/index/getH5InfoByRoom?room_id={}",
            self.sroomid,
        )
    }
}

#[derive(Clone, Deserialize)]
pub struct ExtRoomInfo {
    pub watched_show: WatchedInfo,
}

#[derive(Clone, Deserialize)]
pub struct WatchedInfo {
    pub switch: bool,
    pub num: u32,
}
