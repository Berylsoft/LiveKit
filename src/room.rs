use rocksdb::DB;
use crate::{
    config::{STORAGE_VERSION, EVENT_CHANNEL_BUFFER_SIZE, RoomConfig, GeneralConfig},
    rest::room::RoomInfo,
    client::{channel, Sender, Receiver},
};

pub struct Room {
    pub roomid: u32,
    pub info: RoomInfo,
    pub storage: DB,
    pub channel_sender: Sender, 
}

impl Room {
    pub async fn init(room_config: &RoomConfig, general_config: &GeneralConfig) -> Self {
        let info = RoomInfo::call(room_config.roomid).await.unwrap();
        let roomid = info.room_id;
        let storage_name = match &room_config.alias {
            None => format!("{}-{}", roomid, STORAGE_VERSION),
            Some(alias) => format!("{}-{}", alias, STORAGE_VERSION),
        };
        let storage = DB::open_default(format!("{}/{}", general_config.storage_root, storage_name)).unwrap();
        let (channel_sender, _) = channel(EVENT_CHANNEL_BUFFER_SIZE);

        Room {
            roomid,
            info,
            storage,
            channel_sender,
        }
    }

    pub async fn update_info(&mut self) {
        self.info = RoomInfo::call(self.roomid).await.unwrap();
    }

    pub fn receive(&self) -> Receiver {
        self.channel_sender.subscribe()
    }
}
