use rocksdb::DB;
use crate::{
    config::{STORAGE_VERSION, EVENT_CHANNEL_BUFFER_SIZE, RoomConfig, GroupConfig},
    rest::room::RoomInfo,
    client::{channel, Sender, Receiver},
};

pub struct Room {
    roomid: u32,
    info: RoomInfo,
    channel_sender: Sender,
}

impl Room {
    pub async fn init(room_config: &RoomConfig, group_config: &GroupConfig) -> (Self, DB) {
        let info = RoomInfo::call(room_config.roomid).await.unwrap();
        let roomid = info.room_id;
        let storage_name = format!(
            "{}-{}",
            match &room_config.alias {
                None => roomid.to_string(),
                Some(alias) => alias.clone(),
            },
            STORAGE_VERSION,
        );
        let storage = DB::open_default(format!("{}/{}", group_config.storage_root, storage_name)).unwrap();
        let (channel_sender, _) = channel(EVENT_CHANNEL_BUFFER_SIZE);

        (
            Room {
                roomid,
                info,
                channel_sender,
            },
            storage,
        )
    }

    pub fn id(&self) -> u32 {
        self.roomid
    }

    pub fn info(&self) -> RoomInfo {
        self.info.clone()
    }

    pub async fn update_info(&mut self) {
        self.info = RoomInfo::call(self.roomid).await.unwrap();
    }

    pub fn sender(&self) -> Sender {
        self.channel_sender.clone()
    }

    pub fn receiver(&self) -> Receiver {
        self.channel_sender.subscribe()
    }
}
