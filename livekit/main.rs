use std::path::PathBuf;
use rand::{Rng, thread_rng as rng};
use structopt::StructOpt;
use tokio::{signal, fs};
use tiny_tokio_actor::*;
use livekit::{config::*, GlobalEvent, group::{self, Group}};

#[derive(StructOpt)]
struct Args {
    #[structopt(short = "c", long, parse(from_os_str))]
    config_path: PathBuf,
}

#[tokio::main]
async fn main() {
    env_logger::init();

    let config = fs::read_to_string(Args::from_args().config_path).await.expect("loading config error");
    let GlobalConfig { group } = toml::from_str(config.as_str()).expect("parsing config error");

    let system = ActorSystem::new("system", EventBus::<GlobalEvent>::new(1000));

    let mut group = group.into_iter();
    let GroupConfig { config, rooms } = group.next().unwrap();
    matches!(group.next(), None);

    let group = Group::new(config).await;
    let group_handle = system.create_actor(
        format!("group-{}", rng().gen::<u128>()).as_str(),
        group
    ).await.unwrap();
    group_handle.ask(group::command::AddRooms { msroomids: rooms }).await.unwrap().unwrap();
    println!("{}", group_handle.ask(group::command::DumpStatus).await.unwrap().unwrap());
    println!("{:?}", group_handle.ask(group::command::DumpConfig).await.unwrap().unwrap());

    signal::ctrl_c().await.unwrap();
    group_handle.ask(group::command::CloseAll).await.unwrap().unwrap();
}
