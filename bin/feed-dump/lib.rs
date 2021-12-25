use std::convert::TryInto;
use serde_json::Value as JsonValue;
use livekit_feed_client::{util::Timestamp, package::Package};

#[derive(serde::Serialize)]
pub struct Record {
    time: u64,
    payloads: JsonValue,
}

pub fn record<T: AsRef<[u8]>>(k: T, v: T) -> String {
    let packages = Package::decode(v.as_ref()).flatten();
    let payloads = if packages.len() == 1 {
        packages.into_iter().next().unwrap().to_json().unwrap()
    } else {
        let payloads: Vec<JsonValue> = packages.into_iter().map(|package| package.to_json().unwrap()).collect();
        serde_json::to_value(payloads).unwrap()
    };
    serde_json::to_string(&Record {
        time: Timestamp::from_bytes(k.as_ref().try_into().unwrap()).digits(),
        payloads,
    }).unwrap()
}
