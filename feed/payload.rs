use crate::util::*;

#[derive(Debug)]
pub struct Key {
    time: u64,
    hash: u32,
}

impl Key {
    pub fn encode(&self) -> Box<[u8]> {
        [
            self.time.to_be_bytes().as_slice(),
            self.hash.to_be_bytes().as_slice(),
        ].concat().into_boxed_slice()
    }

    pub fn decode(raw: &[u8]) -> Key {
        assert_eq!(raw.len(), 12);
        let (time, hash) = raw.split_at(8);
        Key {
            time: u64::from_be_bytes(time.try_into().unwrap()),
            hash: u32::from_be_bytes(hash.try_into().unwrap()),
        }
    }
}

pub struct Payload {
    pub time: u64,
    pub payload: Box<[u8]>,
}

impl Payload {
    pub fn new(payload: Box<[u8]>) -> Payload {
        Payload {
            time: now(),
            payload,
        }
    }

    pub fn from_kv(key: Box<[u8]>, value: Box<[u8]>) -> Payload {
        let Key { time, hash } = Key::decode(&key);
        assert_eq!(hash, crc32(&value));
        Payload {
            time,
            payload: value,
        }
    }

    pub fn from_nonhash_kv(key: Box<[u8]>, value: Box<[u8]>) -> Payload {
        assert_eq!(key.len(), 8);
        Payload {
            time: u64::from_be_bytes(key.as_ref().try_into().unwrap()),
            payload: value,
        }
    }

    pub fn get_key(&self) -> Key {
        Key {
            time: self.time,
            hash: crc32(self.payload.as_ref()),
        }
    }
}
