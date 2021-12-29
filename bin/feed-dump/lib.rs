use std::convert::TryInto;
use serde_json::Value as JsonValue;
use livekit_feed::{util::Timestamp, package::Package};

#[derive(serde::Serialize)]
pub struct Record {
    time: u64,
    payloads: JsonValue,
}

pub fn record<T: AsRef<[u8]>>(k: T, v: T) -> String {
    serde_json::to_string(&Record {
        time: Timestamp::from_bytes(k.as_ref().try_into().unwrap()).digits(),
        payloads: Package::decode(v.as_ref()).unwrap().into_json().unwrap(),
    }).unwrap()
}
