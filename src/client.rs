use rand::{seq::SliceRandom, thread_rng as rng};
use tokio::{spawn, sync::broadcast::Sender, time::{self, Duration}};
use futures_util::{StreamExt, SinkExt};
use tokio_tungstenite::{connect_async, tungstenite::{self, protocol::Message}};
use rocksdb::DB;
use crate::{package::Package, util::Timestamp, rest::HostsInfo};

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
            roomid,
            url: format!("wss://{}:{}/sub", host.host, host.wss_port),
            key: hosts_info.token,
        })
    }
}

#[derive(Clone, Debug)]
pub enum Event {
    Open,
    Close,
    Popularity(u32),
    Message(String),
}

pub async fn repeater(roomid: u32, channel_tx: &mut Sender<Event>, storage: &DB) -> Result<(), tungstenite::Error> {
    let connection = Connect::new(roomid).await.unwrap();
    let (socket, _) = connect_async(connection.url.as_str()).await.unwrap();
    let (mut socket_tx, mut socket_rx) = socket.split();

    let init = Message::Binary(Package::create_init_request(&connection).encode());
    socket_tx.send(init).await.unwrap();
    println!("> init sent");

    spawn(async move {
        let heartbeat = Message::Binary(Package::HeartbeatRequest().encode());
        let mut interval = time::interval(Duration::from_secs(30));
        loop {
            interval.tick().await;
            socket_tx.send(heartbeat.clone()).await.unwrap();
            println!("> heartbeat sent");
        }
    });

    loop {
        for maybe_message in socket_rx.next().await {
            match maybe_message? {
                Message::Binary(payload) => {
                    println!("> received");
                    storage.put(Timestamp::now().to_bytes(), &payload).unwrap();
                    for event in Package::decode(&payload).into_events() {
                        channel_tx.send(event).unwrap();
                    }
                },
                _ => panic!("unexpected received websocket message type"),
            }
        }
    }
}
