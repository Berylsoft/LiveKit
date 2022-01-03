pub fn now() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis().try_into().unwrap()
}

pub fn crc32(raw: &[u8]) -> u32 {
    let mut hasher = crc32fast::Hasher::new();
    hasher.update(raw);
    hasher.finalize()
}

pub type BareKeyWithHash = [u8; 12];
pub type BareKeyWithoutHash = [u8; 8];

pub enum Key {
    WithHash(BareKeyWithHash),
    WithoutHash(BareKeyWithoutHash),
}

impl Key {
    pub fn from(raw: &[u8]) -> Option<Key> {
        let len = raw.len();
        if len == 12 {
            Some(Key::WithHash(raw.try_into().unwrap()))
        } else if len == 8 {
            Some(Key::WithoutHash(raw.try_into().unwrap()))
        } else {
            None
        }
    }

    pub fn as_slice(&self) -> &[u8] {
        match self {
            Key::WithHash(k) => k.as_slice(),
            Key::WithoutHash(k) => k.as_slice(),
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
        Payload {
            time: u64::from_be_bytes(Key::from(k.as_ref()).unwrap().as_slice()[0..8].try_into().unwrap()),
            payload: v.as_ref().to_vec(),
        }
    }

    pub fn get_bare_key(&self) -> BareKeyWithHash {
        [
            self.time.to_be_bytes().as_slice(),
            crc32(self.payload.as_slice()).to_be_bytes().as_slice(),
        ].concat().try_into().unwrap()
    }

    pub fn get_key(&self) -> Key {
        Key::WithHash(self.get_bare_key())
    }

    pub fn check_key(&self, key: Key) -> bool {
        match key {
            Key::WithHash(k) => k == self.get_bare_key(),
            Key::WithoutHash(_) => true,
        }
    }
}
