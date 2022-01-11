use tiny_tokio_actor::*;
use livekit_api::{client::HttpClient, info::{RoomInfo, UserInfo}};
use crate::{config::*, GlobalEvent, group::Group};

pub mod command {
    use livekit_api::info::{RoomInfo, UserInfo};

    pub type CommandResult<T> = Result<T, String>;

    command! { GetInfo => (RoomInfo, UserInfo) }
    command! { UpdateInfo => () }
}

pub struct InfoUpdater {
    roomid: u32,
    http_client2: HttpClient,
    currect: (RoomInfo, UserInfo),
}

impl InfoUpdater {
    pub async fn new(roomid: u32, group: &Group, info: RoomInfo) -> InfoUpdater {
        let http_client2 = group.http_client2.clone();
        let user_info = UserInfo::call(&http_client2, roomid).await.unwrap();

        InfoUpdater {
            roomid,
            http_client2,
            currect: (info, user_info),
        }
    }

    pub fn close(self) {
        println!("{} InfoUpdater now off", self.roomid);
    }
}

impl Actor<GlobalEvent> for InfoUpdater {}

#[async_trait]
impl Handler<GlobalEvent, command::GetInfo> for InfoUpdater {
    async fn handle(&mut self, _cmd: command::GetInfo, _ctx: &mut ActorContext<GlobalEvent>) -> <command::GetInfo as Message>::Response {
        Ok(self.currect.clone())
    }
}
