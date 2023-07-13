// region: log_config

mod log_config {
    use std::path::PathBuf;
    use log::LevelFilter;
    use log4rs::{
        append::file::FileAppender,
        config::{Appender, Config, Root},
        encode::json::JsonEncoder,
    };

    pub fn log_config(path: PathBuf, debug: bool) -> Config {
        Config::builder()
            .appender(
                Appender::builder().build(
                    "logfile",
                    Box::new(
                        FileAppender::builder()
                            .encoder(Box::new(JsonEncoder::new()))
                            .build(path)
                            .unwrap(),
                    ),
                ),
            )
            .build(
                Root::builder()
                    .appender("logfile")
                    .build(if debug { LevelFilter::Debug } else { LevelFilter::Info }),
            )
            .unwrap()
    }
}

use log_config::log_config;

// endregion

// region: brapi_live::feed

mod api {
    use brapi_model::{*, prelude::*};

    #[derive(Clone, Debug, Serialize)]
    pub struct GetHostsInfo {
        #[serde(rename = "id")]
        pub roomid: u32,
    }

    #[derive(Clone, Debug, Deserialize)]
    pub struct HostsInfo {
        pub host_list: Vec<HostInfo>,
        pub token: String,
    }

    #[derive(Clone, Debug, Deserialize)]
    pub struct HostInfo {
        pub host: String,
        pub port: u16,
        pub ws_port: u16,
        pub wss_port: u16,
    }

    impl RestApi for GetHostsInfo {
        const BIZ: BizKind = BizKind::Live;
        const METHOD: RestApiRequestMethod = RestApiRequestMethod::BareGet;
        const PATH: &'static str = "/xlive/web-room/v1/index/getDanmuInfo";
        const DEFAULT: Option<&'static str> = Some("type=0");
        type Response = HostsInfo;
    }
}

use api::GetHostsInfo;

// endregion

use std::path::PathBuf;
use rand::{seq::SliceRandom, thread_rng as rng};

use futures_util::{Future, StreamExt};
use tokio::{spawn, signal, time::{sleep, Duration}, fs};

use brapi_client::{client::Client, access::Access};
use livekit_feed_stor_raw::Writer;
use livekit_feed::stream::{FeedStream, INIT_INTERVAL_MS, INIT_RETRY_INTERVAL_SEC, RETRY_INTERVAL_MS};

// region: rec

macro_rules! unwrap_or_continue {
    ($res:expr, $or:expr) => {
        match $res {
            Ok(val) => val,
            Err(err) => {
                $or(err);
                sleep(Duration::from_secs(INIT_RETRY_INTERVAL_SEC)).await;
                continue;
            }
        }
    };
}

fn rec(roomid: u32, api_client: &Client, writer: &Writer) -> impl Future<Output = ()> {
    let api_client = api_client.clone();
    let room_writer = writer.open_room(roomid);

    async move {
        loop {
            let hosts_info = unwrap_or_continue!(
                api_client.call(&GetHostsInfo { roomid }).await,
                |err| log::warn!("[{: >10}] get hosts error {:?}", roomid, err)
            );

            let host = hosts_info.host_list.choose(&mut rng()).expect("FATAL: empty host list");
            let mut stream = unwrap_or_continue!(
                FeedStream::connect_ws(&host.host, host.wss_port, roomid, api_client.uid().unwrap(), api_client.devid3().unwrap(), hosts_info.token).await,
                |err| log::warn!("[{: >10}] error during connecting {:?}", roomid, err)
            );

            log::info!("[{: >10}] open", roomid);

            while let Some(may_payload) = stream.next().await {
                if let Some(payload) = may_payload {
                    if let Err(msg) = room_writer.insert_payload(&payload).await {
                        panic!("{}", msg);
                    }
                }
            }

            log::info!("[{: >10}] close", roomid);

            sleep(Duration::from_millis(RETRY_INTERVAL_MS)).await;
        }
    }
}

// endregion

/// Berylsoft LiveKit feedrec
#[derive(argh::FromArgs)]
struct Args {
    /// comma-separated list of roomid (no short id)
    #[argh(option, short = 'r')]
    roomid_list: String,
    /// feed raw storage directory path
    #[argh(option, short = 's')]
    stor_path: PathBuf,
    /// log file path and name
    #[argh(option, short = 'l')]
    log_path: Option<PathBuf>,
    /// access path
    #[argh(option, short = 'a')]
    access_path: PathBuf,
    /// set log level to debug (default is info)
    #[argh(switch)]
    log_debug: bool,
}

#[tokio::main]
async fn main() {
    let Args { roomid_list, stor_path, log_path, access_path, log_debug } = argh::from_env();
    if let Some(log_path) = log_path {
        log4rs::init_config(log_config(log_path, log_debug)).expect("FATAL: error during init logger");
    }
    let access = fs::read(access_path).await.unwrap();
    let access: Access = serde_json::from_slice(&access).unwrap();
    let (writer, writer_close) = Writer::open(stor_path).await.expect("FATAL: error during init feed raw storage");
    let api_client = Client::new(Some(access), None);
    for roomid in roomid_list.split(',').map(|roomid| roomid.parse::<u32>().expect("FATAL: invaild roomid")) {
        spawn(rec(roomid, &api_client, &writer));
        sleep(Duration::from_millis(INIT_INTERVAL_MS)).await;
    }
    signal::ctrl_c().await.expect("FATAL: error during setting ctrl-c listener");
    writer_close.close_and_wait().await.expect("FATAL: Error occurred during closing");
}
