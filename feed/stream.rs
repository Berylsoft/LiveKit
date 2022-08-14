use std::{pin::Pin, task::{Context, Poll}};
use futures::{Stream, StreamExt, SinkExt, ready};
use tokio::{spawn, time::{self, Duration}, net::TcpStream};
// for TcpFeedStream
use tokio::{io::{Error as IoError, AsyncRead, AsyncWriteExt, ReadBuf}, net::tcp::OwnedReadHalf as TcpStreamRx};
// for WsFeedStream
use tokio_tungstenite::{connect_async as connect_ws_stream, tungstenite::{protocol::Message, Error as WsError, http::Uri}};
use crate::{package::Package, schema::InitRequest};

// for FeedStream
pub const HEARTBEAT_RATE_SEC: u64 = 30;
pub const INIT_INTERVAL_MS: u64 = 100;
pub const TCP_BUFFER_SIZE: usize = 1024 * 8;

// for outer control flow
pub const RETRY_INTERVAL_MS: u64 = 5000;
pub const INIT_RETRY_INTERVAL_SEC: u64 = 5;

fn now() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis().try_into().unwrap()
}

pub struct Payload {
    pub time: u64,
    pub payload: Box<[u8]>,
}

impl Payload {
    pub fn new_now(payload: Box<[u8]>) -> Payload {
        Payload {
            time: now(),
            payload,
        }
    }
}

#[inline]
fn create_init_request(roomid: u32, token: String) -> Package {
    Package::InitRequest(serde_json::to_string(&InitRequest::new_web_without_uid(roomid, token)).unwrap())
}

pub struct FeedStream<T> {
    roomid: u32,
    rx: T,
}

type WsStreamRx = futures::stream::SplitStream<tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<TcpStream>>>;
pub type WsFeedStream = FeedStream<WsStreamRx>;

#[inline]
fn create_ws_url(host: String, port: u16) -> Uri {
    Uri::builder()
        .scheme("wss")
        .authority(concat_string::concat_string!(host, port.to_string()))
        .path_and_query("/sub")
        .build().unwrap()
}

#[inline]
fn wrap_ws_message(bytes: Box<[u8]>) -> Message {
    Message::Binary(bytes.to_vec())
}

impl WsFeedStream {
    pub async fn connect_ws(host: String, port: u16, roomid: u32, token: String) -> Result<WsFeedStream, WsError> {
        let (stream, _) = connect_ws_stream(create_ws_url(host, port)).await?;
        let (mut tx, rx) = stream.split();
        log::debug!("[{: >10}] (ws) connected", roomid);

        let init = wrap_ws_message(create_init_request(roomid, token).encode().unwrap());
        tx.send(init).await?;
        log::debug!("[{: >10}] (ws) sent: init", roomid);

        spawn(async move {
            let heartbeat = wrap_ws_message(Package::HeartbeatRequest.encode().unwrap());
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
}

impl Stream for WsFeedStream {
    type Item = Option<Payload>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        Poll::Ready(match ready!(Pin::new(&mut self.rx).poll_next(cx)) {
            Some(Ok(message)) => Some(match message {
                Message::Binary(payload) => {
                    log::debug!("[{: >10}] (ws) recv: message {}", self.roomid, payload.len());
                    Some(Payload::new_now(payload.into_boxed_slice()))
                },
                Message::Ping(payload) => {
                    if payload.is_empty() {
                        log::debug!("[{: >10}] (ws) recv: empty ping", self.roomid);
                    } else {
                        log::error!("[{: >10}] (ws) recv: non-empty ping {:?}", self.roomid, payload);
                    }
                    None
                },
                message => {
                    log::error!("[{: >10}] (ws) recv: unexpected message type {:?}", self.roomid, message);
                    None
                },
            }),
            Some(Err(error)) => {
                log::warn!("[{: >10}] (ws) close: caused by {:?}", self.roomid, error);
                None
            },
            None => {
                log::warn!("[{: >10}] (ws) close: normally", self.roomid);
                None
            },
        })
    }
}

pub type TcpFeedStream = FeedStream<TcpStreamRx>;

impl TcpFeedStream {
    pub async fn connect_tcp(host: String, port: u16, roomid: u32, token: String) -> Result<TcpFeedStream, IoError> {
        let stream = TcpStream::connect((host, port)).await?;
        let (rx, mut tx) = stream.into_split();
        log::debug!("[{: >10}] (tcp) connected", roomid);

        let init = create_init_request(roomid, token).encode().unwrap();
        tx.write_all(init.as_ref()).await?;
        log::debug!("[{: >10}] (tcp) sent: init", roomid);

        spawn(async move {
            let heartbeat = Package::HeartbeatRequest.encode().unwrap();
            let mut interval = time::interval(Duration::from_secs(HEARTBEAT_RATE_SEC));
            loop {
                interval.tick().await;
                if let Err(error) = tx.write_all(heartbeat.as_ref()).await {
                    log::warn!("[{: >10}] (tcp) stop sending: (heartbeat-thread) caused by {:?}", roomid, error);
                    break;
                }
                log::debug!("[{: >10}] (tcp) sent: (heartbeat-thread) heartbeat", roomid);
            }
        });

        Ok(TcpFeedStream { roomid, rx })
    }
}

impl Stream for TcpFeedStream {
    type Item = Option<Payload>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let mut bytes = [0u8; TCP_BUFFER_SIZE];
        let mut readbuf = ReadBuf::new(&mut bytes);

        Poll::Ready(match ready!(Pin::new(&mut self.rx).poll_read(cx, &mut readbuf)) {
            Ok(()) => Some({
                let payload = readbuf.filled();
                log::debug!("[{: >10}] (tcp) recv: message {}", self.roomid, payload.len());
                Some(Payload::new_now(Box::from(payload)))
            }),
            Err(error) => {
                log::warn!("[{: >10}] (tcp) close: caused by {:?}", self.roomid, error);
                None
            },
        })
    }
}
