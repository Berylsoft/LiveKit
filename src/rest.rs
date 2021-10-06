pub mod room {
    use serde::Deserialize;
    use crate::util::rest::call;

    #[derive(Deserialize)]
    pub struct HostsInfo {
        pub host_list: Vec<HostInfo>,
        pub token: String,
    }

    #[derive(Deserialize)]
    pub struct HostInfo {
        pub host: String,
        // pub port: u16,
        // pub ws_port: u16,
        pub wss_port: u16,
    }

    // TODO (consider) macro
    impl HostsInfo {
        #[inline]
        pub async fn call(roomid: u32) -> Result<Self, String> {
            call(format!(
                "/xlive/web-room/v1/index/getDanmuInfo?id={}&type=0",
                roomid
            )).await
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

    impl RoomInfo {
        #[inline]
        pub async fn call(roomid: u32) -> Result<Self, String> {
            call(format!(
                "/room/v1/Room/get_info?id={}",
                roomid
            )).await
        }
    }

    #[derive(Deserialize)]
    pub struct PlayUrlCodecUrlInfo {
        pub host: String,
        pub extra: String,
        pub stream_ttl: i32,
    }

    #[derive(Deserialize)]
    pub struct PlayUrlCodec {
        pub codec_name: String,
        pub current_qn: i32,
        pub accept_qn: Vec<i32>,
        pub base_url: String,
        pub url_info: Vec<PlayUrlCodecUrlInfo>,
    }

    #[derive(Deserialize)]
    pub struct PlayUrlFormat {
        pub format_name: String,
        pub codec: Vec<PlayUrlCodec>,
    }

    #[derive(Deserialize)]
    pub struct PlayUrlStream {
        pub protocol_name: String,
        pub format: Vec<PlayUrlFormat>,
    }

    #[derive(Deserialize)]
    pub struct PlayUrl {
        pub stream: Vec<PlayUrlStream>,
    }

    #[derive(Deserialize)]
    pub struct PlayUrlInfo {
        pub playurl: PlayUrl,
    }

    #[derive(Deserialize)]
    pub struct PlayInfo {
        pub playurl_info: PlayUrlInfo,
    }

    impl PlayInfo {
        #[inline]
        pub async fn call(roomid: u32, qn: i32) -> Result<Self, String> {
            call(format!(
                "/xlive/web-room/v2/index/getRoomPlayInfo?room_id={}&protocol=0,1&format=0,1,2&codec=0,1&qn={}&platform=web&ptype=8",
                roomid,
                qn,
            )).await
        }
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

    impl UserInfo {
        #[inline]
        pub async fn call(roomid: u32) -> Result<Self, String> {
            call(format!(
                "/live_user/v1/UserInfo/get_anchor_in_room?roomid={}",
                roomid,
            )).await
        }
    }
}
