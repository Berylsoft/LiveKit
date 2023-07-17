use bytes::Bytes;
use futures_util::{StreamExt, SinkExt};
use tokio::{spawn, time::{self, Duration}, net::TcpStream};
// for TcpFeedStream
use tokio::{io::{Error as IoError, AsyncReadExt, AsyncWriteExt}, net::tcp::OwnedReadHalf as TcpStreamRx};
// for WsFeedStream
use tokio_tungstenite::{connect_async as connect_ws_stream, tungstenite::{protocol::Message, Error as WsError, http::Uri}};
use crate::{package::Package, schema::InitRequest};

// for FeedStream
pub const HEARTBEAT_RATE_SEC: u64 = 30;
pub const TCP_BUFFER_SIZE: usize = 1024 * 8;

// for outer control flow
pub const RETRY_INTERVAL_MS: u64 = 5000;
pub const INIT_INTERVAL_MS: u64 = 100;
pub const INIT_RETRY_INTERVAL_SEC: u64 = 5;

pub fn now() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis().try_into().unwrap()
}

pub struct Payload {
    pub time: u64,
    pub payload: Bytes,
}

impl Payload {
    pub fn new_now(payload: Vec<u8>) -> Payload {
        Payload {
            time: now(),
            payload: payload.into(),
        }
    }
}

#[inline]
fn create_init_request(roomid: u32, uid: u64, devid3: String, token: String) -> Package {
    Package::InitRequest(serde_json::to_string(&InitRequest::new_v3_web_with_access(roomid, uid, devid3, token)).unwrap())
}

pub struct FeedStream<T> {
    roomid: u32,
    rx: T,
}

type WsStreamRx = futures_util::stream::SplitStream<tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<TcpStream>>>;
pub type WsFeedStream = FeedStream<WsStreamRx>;

#[inline]
fn create_ws_url(host: &str, port: u16) -> Uri {
    Uri::builder()
        .scheme("wss")
        .authority(format!("{host}:{port}"))
        .path_and_query("/sub")
        .build().unwrap()
}

impl WsFeedStream {
    pub async fn connect_ws(host: &str, port: u16, roomid: u32, uid: u64, devid3: String, token: String) -> Result<WsFeedStream, WsError> {
        let (stream, _) = connect_ws_stream(create_ws_url(host, port)).await?;
        let (mut tx, rx) = stream.split();
        log::debug!("[{: >10}] (ws) connected", roomid);

        let init = Message::Binary(create_init_request(roomid, uid, devid3, token).encode().unwrap());
        tx.send(init).await?;
        log::debug!("[{: >10}] (ws) sent: init", roomid);

        spawn(async move {
            let heartbeat = Message::Binary(Package::HeartbeatRequest.encode().unwrap());
            let mut interval = time::interval(Duration::from_secs(HEARTBEAT_RATE_SEC));
            loop {
                interval.tick().await;
                if let Err(error) = tx.send(heartbeat.clone()).await {
                    log::warn!("[{: >10}] (ws) stop sending: (heartbeat-thread) caused by {:?}", roomid, error);
                    break;
                }
                log::debug!("[{: >10}] (ws) sent: (heartbeat-thread) heartbeat", roomid);
            }
        });

        Ok(WsFeedStream { roomid, rx })
    }

    pub async fn recv(&mut self) -> Option<Payload> {
        loop {
            match self.rx.next().await {
                Some(Ok(message)) => match message {
                    Message::Binary(payload) => {
                        log::debug!("[{: >10}] (ws) recv: message {}", self.roomid, payload.len());
                        return Some(Payload::new_now(payload.into()));
                    },
                    Message::Ping(payload) => {
                        if payload.is_empty() {
                            log::debug!("[{: >10}] (ws) recv: empty ping", self.roomid);
                        } else {
                            log::error!("[{: >10}] (ws) recv: non-empty ping {:?}", self.roomid, payload);
                        }
                        continue;
                    },
                    message => {
                        log::error!("[{: >10}] (ws) recv: unexpected message type {:?}", self.roomid, message);
                        continue;
                    },
                },
                Some(Err(error)) => {
                    log::warn!("[{: >10}] (ws) close: caused by {:?}", self.roomid, error);
                    return None;
                },
                None => {
                    log::warn!("[{: >10}] (ws) close: normally", self.roomid);
                    return None;
                },
            }
        }
    }
}

pub type TcpFeedStream = FeedStream<TcpStreamRx>;

impl TcpFeedStream {
    pub async fn connect_tcp(host: &str, port: u16, roomid: u32, uid: u64, devid3: String, token: String) -> Result<TcpFeedStream, IoError> {
        let stream = TcpStream::connect((host, port)).await?;
        let (rx, mut tx) = stream.into_split();
        log::debug!("[{: >10}] (tcp) connected", roomid);

        let init = create_init_request(roomid, uid, devid3, token).encode().unwrap();
        tx.write_all(&init).await?;
        log::debug!("[{: >10}] (tcp) sent: init", roomid);

        spawn(async move {
            let heartbeat = Package::HeartbeatRequest.encode().unwrap();
            let mut interval = time::interval(Duration::from_secs(HEARTBEAT_RATE_SEC));
            loop {
                interval.tick().await;
                if let Err(error) = tx.write_all(&heartbeat).await {
                    log::warn!("[{: >10}] (tcp) stop sending: (heartbeat-thread) caused by {:?}", roomid, error);
                    break;
                }
                log::debug!("[{: >10}] (tcp) sent: (heartbeat-thread) heartbeat", roomid);
            }
        });

        Ok(TcpFeedStream { roomid, rx })
    }

    pub async fn recv(&mut self) -> Option<Payload> {
        let log_error = |error: IoError| {
            log::warn!("[{: >10}] (tcp) close: caused by {:?}", self.roomid, error);
        };

        // TODO avoid copy with "peek_exact"
        let mut len_buf = [0; 4];
        self.rx.read_exact(&mut len_buf).await.map_err(log_error).ok()?;
        let len = u32::from_be_bytes(len_buf).try_into().unwrap();
        let mut payload = vec![0; len];
        payload[0..4].copy_from_slice(&len_buf);
        self.rx.read_exact(&mut payload[4..]).await.map_err(log_error).ok()?;
        log::debug!("[{: >10}] (tcp) recv: message {}", self.roomid, len);
        Some(Payload::new_now(payload))
    }
}
