pub mod compress {
    pub use inflate::inflate_bytes as inflate;

    use std::io::Cursor;
    use brotli_decompressor::BrotliDecompress;

    pub fn brotli(raw: &[u8]) -> Result<Vec<u8>, std::io::Error> {
        let mut decoded = Vec::new();
        BrotliDecompress(&mut Cursor::new(raw), &mut Cursor::new(&mut decoded))?;
        Ok(decoded)
    }
}
