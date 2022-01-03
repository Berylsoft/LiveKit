pub fn now() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis().try_into().unwrap()
}

pub fn crc32(raw: &[u8]) -> u32 {
    let mut hasher = crc32fast::Hasher::new();
    hasher.update(raw);
    hasher.finalize()
}

pub struct KeyWithHash(u64, u32);

impl KeyWithHash {
    pub fn encode(&self) -> Vec<u8> {
        [
            self.0.to_be_bytes().as_slice(),
            self.1.to_be_bytes().as_slice(),
        ].concat()
    }
}

pub enum Key {
    WithHash(KeyWithHash),
    WithoutHash(u64),
}

impl Key {
    pub fn from(raw: &[u8]) -> Option<Key> {
        let len = raw.len();
        if len == 12 {
            let (time, hash) = raw.split_at(8);
            Some(Key::WithHash(KeyWithHash(
                u64::from_be_bytes(time.try_into().unwrap()),
                u32::from_be_bytes(hash.try_into().unwrap()),
            )))
        } else if len == 8 {
            Some(Key::WithoutHash(
                u64::from_be_bytes(raw.try_into().unwrap())
            ))
        } else {
            None
        }
    }
}

pub struct Payload {
    pub time: u64,
    pub payload: Vec<u8>,
}

impl Payload {
    pub fn new(payload: Vec<u8>) -> Payload {
        Payload {
            time: now(),
            payload,
        }
    }

    pub fn from_kv<T: AsRef<[u8]>>(k: T, v: T) -> Payload {
        let v = v.as_ref();

        Payload {
            time: match Key::from(k.as_ref()).unwrap() {
                Key::WithHash(KeyWithHash(time, hash)) => {
                    assert_eq!(hash, crc32(v));
                    time
                },
                Key::WithoutHash(time) => time,
            },
            payload: v.to_vec(),
        }
    }

    pub fn get_key(&self) -> KeyWithHash {
        KeyWithHash(self.time, crc32(self.payload.as_slice()))
    }
}
