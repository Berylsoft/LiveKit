use rand::{seq::SliceRandom, thread_rng as rng};
use tokio::{spawn, sync::broadcast, time::{self, Duration}};
use futures_util::{StreamExt, SinkExt};
use tokio_tungstenite::{connect_async, tungstenite::{self, protocol::Message}};
use rocksdb::DB;
use crate::{config::HEARTBEAT_RATE_SEC, util::Timestamp, package::Package, rest::room::HostsInfo};

pub struct Connect {
    pub roomid: u32,
    pub url: String,
    pub key: String,
}

impl Connect {
    pub async fn new(roomid: u32) -> Self {
        let hosts_info = HostsInfo::call(roomid).await.unwrap();
        let host = &hosts_info.host_list.choose(&mut rng()).unwrap();
        Connect {
            roomid,
            url: format!("wss://{}:{}/sub", host.host, host.wss_port),
            key: hosts_info.token,
        }
    }
}

#[derive(Clone, Debug)]
pub enum Event {
    Open,
    Close,
    Popularity(u32),
    Message(String),
}

pub type Sender = broadcast::Sender<Event>;
pub type Receiver = broadcast::Receiver<Event>;
pub use broadcast::channel;

pub async fn repeater(roomid: u32, channel_sender: &mut Sender, storage: &DB) -> Result<(), tungstenite::Error> {
    let connection = Connect::new(roomid).await;
    let (socket, _) = connect_async(connection.url.as_str()).await.unwrap();
    let (mut socket_sender, mut socket_receiver) = socket.split();

    let init = Message::Binary(Package::create_init_request(&connection).encode());
    socket_sender.send(init).await.unwrap();
    eprintln!("> init sent");

    spawn(async move {
        let heartbeat = Message::Binary(Package::HeartbeatRequest().encode());
        let mut interval = time::interval(Duration::from_secs(HEARTBEAT_RATE_SEC));
        loop {
            interval.tick().await;
            socket_sender.send(heartbeat.clone()).await.unwrap();
            eprintln!("> heartbeat sent");
        }
    });

    loop {
        for maybe_message in socket_receiver.next().await {
            match maybe_message? {
                Message::Binary(payload) => {
                    eprintln!("> received");
                    storage.put(Timestamp::now().to_bytes(), &payload).unwrap();
                    Package::decode(&payload).send_as_events(channel_sender);
                },
                _ => panic!("unexpected received websocket message type"),
            }
        }
    }
}
