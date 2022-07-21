use tokio::{io::AsyncWriteExt, fs::OpenOptions};
use futures::Future;
use livekit_feed::schema::Event as FeedEvent;
use crate::{config::DumpKind, room::{Room, Event}};

pub fn write(kind: &DumpKind, event: &FeedEvent) -> String {
    let mut output = String::new();
    if let DumpKind::Debug = kind {} else {
        if let FeedEvent::Unimplemented { .. } | FeedEvent::Ignored { .. } = event {} else {
            output.push_str(match kind {
                DumpKind::Debug => {
                    format!("{:?}", event)
                },
                DumpKind::NdJson => {
                    serde_json::to_string(event).unwrap()
                },
            }.as_str());
        }
    }
    output.push_str("\n");
    output
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
                    file.write_all(write(&kind, &event).as_bytes()).await.expect("writing to dump file error");
                }
            })
        } else {
            None
        }
    }
}
