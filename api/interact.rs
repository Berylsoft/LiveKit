use serde::{Serialize, Deserialize};
use crate::client::{HttpClient, RestApiResult};
use crate::client::{RestApi, RestApiRequestKind};

#[derive(Serialize)]
pub struct SendDanmaku {
    pub bubble: i32,
    pub dm_type: Option<i32>, // Some(1) for emoji
    pub msg: String,
    pub color: u32,
    pub mode: i32,
    pub fontsize: u32,
    pub rnd: i64,
    pub roomid: u32,
}

#[derive(Debug, Deserialize)]
pub struct SentDanmaku {
    pub mode_info: SentDanmakuModeInfo,
}

#[derive(Debug, Deserialize)]
pub struct SentDanmakuModeInfo {
    pub mode: i32,
    pub show_player_type: i32,
    pub extra: String,
}

impl SendDanmaku {
    #[inline]
    pub async fn call(&self, client: &HttpClient) -> RestApiResult<SentDanmaku> {
        client.call_post("/msg/send", Some(self)).await
    }

    pub fn new(roomid: u32, msg: String, rnd: i64, emoji: bool) -> SendDanmaku {
        SendDanmaku {
            bubble: 0,
            dm_type: if emoji { Some(1) } else { None },
            msg,
            color: 16777215,
            mode: 1,
            fontsize: 25,
            rnd,
            roomid,
        }
    }
}

impl RestApi for SendDanmaku {
    type Response = SentDanmaku;

    fn kind(&self) -> RestApiRequestKind {
        RestApiRequestKind::PostWithForm
    }

    fn path(&self) -> String {
        "/msg/send".to_owned()
    }
}
