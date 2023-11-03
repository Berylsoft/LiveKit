use std::{path::PathBuf, fs};
use brapi_client::client::Client;
use brapi_cli_live::Request;

/// call interact apis with json from command line
#[derive(argh::FromArgs)]
#[argh(subcommand, name = "interact")]
pub struct Args {
    /// json access file path
    #[argh(option, short = 'a')]
    access_path: Option<PathBuf>,
    #[argh(subcommand)]
    inner: Request,
}

pub async fn main(args: Args) {
    let Args { access_path, inner } = args;
    let access_path = access_path.unwrap_or_else(|| std::env::var_os("BAPI_ACCESS_PATH").unwrap().into());
    let access = fs::read_to_string(access_path).unwrap();
    let client = Client::with_access(access, None).unwrap();
    inner.call(client).await;
}
