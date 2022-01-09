pub mod config;
pub mod command;

use std::collections::HashMap;
use tokio::{fs, time::{sleep, Duration}};
use tiny_tokio_actor::*;
use livekit_api::{client::HttpClient, info::RoomInfo};
// use livekit_feed_storage::{Db, open_db};
use crate::config::*;

#[derive(Clone)]
pub struct GlobalEvent(String);

impl SystemEvent for GlobalEvent {}

#[derive(Debug)]
struct RoomHandles {
}

impl RoomHandles {
    fn off(self, roomid: u32) {
        println!("{} handles now off", roomid);
    }
}

pub struct Group {
    pub(crate) rooms: HashMap<u32, Option<RoomHandles>>,
    pub(crate) http_client: HttpClient,
    // pub(crate) http_client2: HttpClient,
    // pub(crate) db: Db,
    pub(crate) config: Config,
}

impl Group {
    pub async fn new(config: Config) -> Group {
        if let Some(dump_config) = &config.dump {
            fs::create_dir_all(&dump_config.path).await.expect("creating dump directory error");
        }
        if let Some(record_config) = &config.record {
            fs::create_dir_all(&record_config.path).await.expect("creating record directory error");
        }

        Group {
            rooms: HashMap::new(),
            http_client: match config.http.clone() {
                Some(HttpConfig { access, proxy }) => HttpClient::new(access, proxy).await,
                None => HttpClient::new(None, None).await,
            },
            // http_client2: HttpClient::new_bare().await,
            // db: open_db(&config.storage.path).expect("opening storage error"),
            config,
        }
    }

    async fn spawn_handles(&self, roomid: u32, _info: RoomInfo) -> RoomHandles {
        println!("{} handles now on", roomid);
        RoomHandles { }
    }
}

impl Actor<GlobalEvent> for Group {}

#[async_trait]
impl Handler<GlobalEvent, command::AddRooms> for Group {
    async fn handle(&mut self, cmd: command::AddRooms, _ctx: &mut ActorContext<GlobalEvent>) -> <command::AddRooms as Message>::Response {
        for msroomid in cmd.msroomids {
            let (sroomid, on): (u32, bool) = if msroomid < 0 {
                ((-msroomid).try_into().unwrap(), false)
            } else {
                (msroomid.try_into().unwrap(), true)
            };
            let info = RoomInfo::call(&self.http_client, sroomid).await.unwrap();
            let roomid = info.room_id;
            match (self.rooms.get(&roomid), on) {
                (None, true) | (Some(None), true) => {
                    let handles = self.spawn_handles(roomid, info).await;
                    self.rooms.insert(roomid, Some(handles));
                },
                (None, false) => {
                    self.rooms.insert(roomid, None);
                },
                (Some(Some(_)), false) => {
                    let prev = self.rooms.insert(roomid, None);
                    prev.unwrap().unwrap().off(roomid);
                },
                (Some(None), false) | (Some(Some(_)), true) => { },
            }
            sleep(Duration::from_millis(INIT_INTERVAL_MS)).await;
        }
        Ok(())
    }
}

#[async_trait]
impl Handler<GlobalEvent, command::DumpConfig> for Group {
    async fn handle(&mut self, _cmd: command::DumpConfig, _ctx: &mut ActorContext<GlobalEvent>) -> <command::DumpConfig as Message>::Response {
        Ok(GroupConfig {
            config: self.config.clone(),
            rooms: self.rooms.iter().map(|(roomid, handles)| {
                let roomid: i64 = (*roomid).into();
                match handles {
                    Some(_) => roomid,
                    None => -roomid,
                }
            }).collect()
        })
    }
}

#[async_trait]
impl Handler<GlobalEvent, command::DumpStatus> for Group {
    async fn handle(&mut self, _cmd: command::DumpStatus, _ctx: &mut ActorContext<GlobalEvent>) -> <command::DumpStatus as Message>::Response {
        Ok(format!("{:?}", self.rooms.iter().collect::<Vec<(&u32, &Option<RoomHandles>)>>()))
    }
}

#[async_trait]
impl Handler<GlobalEvent, command::CloseAll> for Group {
    async fn handle(&mut self, _cmd: command::CloseAll, _ctx: &mut ActorContext<GlobalEvent>) -> <command::CloseAll as Message>::Response {
        let rooms = std::mem::replace(&mut self.rooms, HashMap::new());
        for (roomid, handles) in rooms {
            if let Some(handles) = handles {
                handles.off(roomid);
            }
        }
        Ok(())
    }
}
