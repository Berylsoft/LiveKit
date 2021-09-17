use rand::{seq::SliceRandom, thread_rng as rng};
use crate::rest::HostsInfo;

pub struct Connect {
    pub roomid: u32,
    pub url: String,
    pub key: String,
}

impl Connect {
    pub async fn new(roomid: u32) -> Option<Self> {
        let hosts_info = HostsInfo::get(roomid).await?;
        let host = &hosts_info.host_list.choose(&mut rng()).unwrap();
        Some(Connect {
            roomid: roomid,
            url: format!("wss://{}:{}/sub", host.host, host.wss_port),
            key: hosts_info.token,
        })
    }
}
