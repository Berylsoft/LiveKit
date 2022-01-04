use std::{io::Write, fs::File};
use serde::{Serialize, Deserialize};
use crate::schema::Event;

#[derive(Serialize, Deserialize, Clone)]
pub enum OutputKind {
    Debug,
    NdJson,
}

pub fn write(writer: &mut File, kind: &OutputKind, event: &Event) {
    if let OutputKind::Debug = kind {} else {
        if let Event::Unimplemented | Event::Ignored = event {
            return
        }
    }
    match kind {
        OutputKind::Debug => {
            write!(writer, "{:?}", event).unwrap();
        },
        OutputKind::NdJson => {
            serde_json::to_writer(&*writer, event).unwrap();
        },
    }
    writeln!(writer).unwrap();
}
