use std::{io::Write, fs::{File, OpenOptions}};
use futures::Future;
use serde::{Serialize, Deserialize};
use livekit_feed::schema::Event as FeedEvent;
use crate::room::{Room, Event};

#[derive(Serialize, Deserialize, Clone)]
pub enum OutputKind {
    Debug,
    NdJson,
}

pub fn write(writer: &mut File, kind: &OutputKind, event: &FeedEvent) {
    if let OutputKind::Debug = kind {} else {
        if let FeedEvent::Unimplemented | FeedEvent::Ignored = event {
            return;
        }
    }
    match kind {
        OutputKind::Debug => {
            write!(writer, "{:?}", event).expect("writing to dump file error");
        },
        OutputKind::NdJson => {
            serde_json::to_writer(&*writer, event).expect("writing to dump file error");
        },
    }
    writeln!(writer).expect("writing to dump file error");
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
            }).expect("opening dump file error");

            Some(async move {
                while let Ok(Event::Feed(event)) = rx.recv().await {
                    write(&mut file, &kind, &event);
                }
            })
        } else {
            None
        }
    }
}
