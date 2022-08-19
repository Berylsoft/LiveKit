use argh::FromArgs;
use livekit::cmd::feed_dump;

#[derive(FromArgs)]
/// Berylsoft Livekit
struct Args {
    #[argh(subcommand)]
    inner: Commands,
}

#[derive(FromArgs)]
#[argh(subcommand)]
enum Commands {
    FeedDump(feed_dump::Args),
}

#[tokio::main]
async fn main() {
    match argh::from_env::<Args>().inner {
        Commands::FeedDump(args) => feed_dump::main(args),
    }
}
