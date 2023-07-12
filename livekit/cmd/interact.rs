use std::{path::PathBuf, fs};
use brapi_client::{client::Client, access::Access};
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
    let access: Access = serde_json::from_reader(fs::OpenOptions::new().read(true).open(access_path).unwrap()).unwrap();
    let client = Client::new(Some(access), None);
    inner.call(client).await;
}
