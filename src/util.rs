pub mod compress {
    pub use inflate::inflate_bytes as inflate;

    use std::io::Cursor;
    use brotli_decompressor::BrotliDecompress;

    pub fn de_brotli(raw: &[u8]) -> Result<Vec<u8>, std::io::Error> {
        let mut decoded = Vec::new();
        BrotliDecompress(&mut Cursor::new(raw), &mut Cursor::new(&mut decoded))?;
        Ok(decoded)
    }
}

pub struct Timestamp(i64); // u64?

impl Timestamp {
    pub fn now() -> Self {
        Timestamp(chrono::Utc::now().timestamp_millis())
    }

    #[inline]
    pub fn digits(&self) -> i64 {
        self.0
    }

    pub fn to_bytes(&self) -> [u8; 8] {
        self.digits().to_be_bytes()
    }
}

pub mod bytes {
    pub fn concat(mut a: Vec<u8>, mut b: Vec<u8>) -> Vec<u8> {
        a.append(&mut b);
        a
    }
}
