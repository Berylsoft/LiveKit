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

// region: bilibili_restapi_live::feed

mod api {
    use bilibili_restapi_model::{*, prelude::*};

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

use api::{GetHostsInfo, HostInfo};

// endregion

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

fn rec(roomid: u32, http_client: &Client, db: &Writer) -> impl Future<Output = ()> {
    let http_client = http_client.clone();
    let storage = db.open_room(roomid);

    async move {
        loop {
            let hosts_info = unwrap_or_continue!(
                http_client.call(&GetHostsInfo { roomid }).await,
                |err| log::warn!("[{: >10}] get hosts error {:?}", roomid, err)
            );

            let HostInfo { host, wss_port, .. } = hosts_info.host_list.choose(&mut rng()).unwrap();
            let mut stream = unwrap_or_continue!(
                FeedStream::connect_ws(host.to_owned(), *wss_port, roomid, hosts_info.token).await,
                |err| log::warn!("[{: >10}] error during connecting {:?}", roomid, err)
            );

            log::info!("[{: >10}] open", roomid);

            while let Some(may_payload) = stream.next().await {
                if let Some(payload) = may_payload {
                    if let Err(msg) = storage.insert_payload(&payload).await {
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

use std::path::PathBuf;
use structopt::StructOpt;
use rand::{seq::SliceRandom, thread_rng as rng};

use futures::{Future, StreamExt};
use tokio::{spawn, signal, time::{sleep, Duration}};

use bilibili_restapi_client::client::Client;
use livekit_feed_stor_raw::Writer;
use livekit_feed::stream::{FeedStream, INIT_INTERVAL_MS, INIT_RETRY_INTERVAL_SEC, RETRY_INTERVAL_MS};

#[derive(StructOpt)]
struct Args {
    #[structopt(short = "r", long)]
    roomid_list: String,
    #[structopt(short = "s", long, parse(from_os_str))]
    storage_path: PathBuf,
    #[structopt(short = "l", long, parse(from_os_str))]
    log_path: Option<PathBuf>,
    #[structopt(long)]
    log_debug: bool,
}

#[tokio::main]
async fn main() {
    let Args { roomid_list, storage_path, log_path, log_debug } = Args::from_args();
    if let Some(log_path) = log_path {
        log4rs::init_config(log_config(log_path, log_debug)).unwrap();
    }
    let db = Writer::open(storage_path).unwrap();
    let http_client = Client::new_bare();
    for roomid in roomid_list.split(",").map(|roomid| roomid.parse::<u32>().unwrap()) {
        spawn(rec(roomid, &http_client, &db));
        sleep(Duration::from_millis(INIT_INTERVAL_MS)).await;
    }
    signal::ctrl_c().await.unwrap();
}
