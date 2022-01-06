use std::io::{Write, BufWriter};
use tokio::{io::AsyncWriteExt, fs::OpenOptions};
use futures::Future;
use serde::{Serialize, Deserialize};
use livekit_feed::schema::Event as FeedEvent;
use crate::room::{Room, Event};

#[derive(Serialize, Deserialize, Clone)]
pub enum OutputKind {
    Debug,
    NdJson,
}

pub fn write(kind: &OutputKind, event: &FeedEvent) -> Vec<u8> {
    let mut writer = BufWriter::new(Vec::new());
    if let OutputKind::Debug = kind {} else {
        if let FeedEvent::Unimplemented | FeedEvent::Ignored = event {
            return writer.into_inner().unwrap();
        }
    }
    match kind {
        OutputKind::Debug => {
            write!(writer, "{:?}", event).unwrap();
        },
        OutputKind::NdJson => {
            serde_json::to_writer(&mut writer, event).unwrap();
        },
    }
    writeln!(writer).unwrap();
    writer.into_inner().unwrap()
}

impl Room {
    pub async fn dump(&self) -> Option<impl Future<Output = ()>> {
        if let Some(config) = &self.config.dump {
            let kind = config.kind.clone();
            let rx = self.rx.clone();
            let mut file = OpenOptions::new().write(true).create(true).append(true).open({
                let mut path = config.path.clone();
                path.push(format!("{}.txt", self.id()));
                path
            }).await.expect("opening dump file error");

            Some(async move {
                while let Ok(Event::Feed(event)) = rx.recv().await {
                    file.write_all(write(&kind, &event).as_slice()).await.expect("writing to dump file error");
                }
            })
        } else {
            None
        }
    }
}
