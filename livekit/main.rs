use livekit::cmd::*;

/// Berylsoft Livekit
#[derive(argh::FromArgs)]
struct Args {
    #[argh(subcommand)]
    inner: Commands,
}

#[derive(argh::FromArgs)]
#[argh(subcommand)]
#[allow(non_camel_case_types)]
enum Commands {
    feed_dump(feed_dump::Args),
    interact(interact::Args),
}

#[tokio::main]
async fn main() {
    match argh::from_env::<Args>().inner {
        Commands::feed_dump(args) => feed_dump::main(args),
        Commands::interact(args) => interact::main(args).await,
    }
}
