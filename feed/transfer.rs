use std::{io::{Write, Result as IoResult}, fs::File};
use serde::{Serialize, Deserialize};
use crate::schema::Event;

#[derive(Serialize, Deserialize, Clone)]
pub enum OutputKind {
    Debug,
    NdJson,
}

pub fn write(writer: &mut File, kind: &OutputKind, event: &Event) -> IoResult<()> {
    if let OutputKind::Debug = kind {} else {
        if let Event::Unimplemented | Event::Ignored = event {
            return Ok(());
        }
    }
    match kind {
        OutputKind::Debug => {
            write!(writer, "{:?}", event)?;
        },
        OutputKind::NdJson => {
            serde_json::to_writer(&*writer, event)?;
        },
    }
    writeln!(writer)?;
    Ok(())
}
