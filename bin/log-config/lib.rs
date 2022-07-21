pub use log4rs;

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
