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

pub struct Timestamp(i64);

impl Timestamp {
    pub fn now() -> Self {
        Timestamp(chrono::Utc::now().timestamp())
    }

    pub fn to_bytes(&self) -> [u8; 8] {
        self.0.to_be_bytes()
    }
}
