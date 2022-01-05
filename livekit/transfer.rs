use std::{io::Write, fs::File};
use serde::{Serialize, Deserialize};
use livekit_feed::schema::Event;

#[derive(Serialize, Deserialize, Clone)]
pub enum OutputKind {
    Debug,
    NdJson,
}

pub fn write(writer: &mut File, kind: &OutputKind, event: &Event) {
    if let OutputKind::Debug = kind {} else {
        if let Event::Unimplemented | Event::Ignored = event {
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
