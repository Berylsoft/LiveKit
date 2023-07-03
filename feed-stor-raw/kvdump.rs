pub const BS_IDENT: u32 = 0x42650000;

use std::io::{self, Read, Write};
use blake3::{Hasher, OUT_LEN as HASH_LEN};
use foundations::{num_enum, usize_casting::*, error_enum};

// region: util

fn usize_u32(n: usize) -> Result<u32> {
    n.try_into().map_err(|_| Error::TooLongSize { size: usize_u64(n) })
}

macro_rules! check {
    ($l:expr, $r:expr, $varient:expr) => {
        if $l != $r {
            return Err($varient);
        }
    };
}

// endregion

// region: helper traits

trait ReadExt: Read {
    fn read_bytes(&mut self, len: usize) -> Result<Box<[u8]>> {
        let mut buf = vec![0; len];
        self.read_exact(&mut buf)?;
        Ok(buf.into_boxed_slice())
    }

    fn read_bytes_sized<const N: usize>(&mut self) -> Result<[u8; N]> {
        let mut buf = [0; N];
        self.read_exact(&mut buf)?;
        Ok(buf)
    }
    
    #[inline]
    fn read_u8(&mut self) -> Result<u8> {
        self.read_bytes_sized().map(u8::from_be_bytes)
    }

    #[inline]
    fn read_u32(&mut self) -> Result<u32> {
        self.read_bytes_sized().map(u32::from_be_bytes)
    }

    #[inline]
    fn read_hash(&mut self) -> Result<[u8; HASH_LEN]> {
        self.read_bytes_sized()
    }
}

impl<R: Read> ReadExt for R {}

trait WriteExt: Write {
    #[inline]
    fn write_bytes<B: AsRef<[u8]>>(&mut self, bytes: B) -> io::Result<()> {
        self.write_all(bytes.as_ref())
    }

    #[inline]
    fn write_u8(&mut self, val: u8) -> io::Result<()> {
        self.write_bytes(val.to_be_bytes())
    }

    #[inline]
    fn write_u32(&mut self, val: u32) -> io::Result<()> {
        self.write_bytes(val.to_be_bytes())
    }
}

impl<W: Write> WriteExt for W {}

// endregion

// region: row types

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct KV {
    pub scope: Box<[u8]>,
    pub key: Box<[u8]>,
    pub value: Box<[u8]>,
}

pub type Hash = [u8; HASH_LEN];

num_enum! {
    enum RowType {
        KV   = 0,
        Hash = 1,
        End  = 2,
    } as u8 else Error::RowType
}

#[derive(Debug, Clone)]
pub enum Row {
    KV(KV),
    Hash(Hash),
    End,
}

// endregion

// region: config types

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct Sizes {
    pub scope: Option<u32>,
    pub key: Option<u32>,
    pub value: Option<u32>,
}

impl Sizes {
    fn flag(&self) -> u8 {
        let mut flag = 0;
        macro_rules! skv_op_impl {
            ($($x:ident,)*) => {$(
                if self.$x.is_some() {
                    flag |= SIZES_FLAG_BASES.$x;
                }
            )*};
        }
        skv_op_impl!(scope, key, value,);
        flag
    }
}

struct SizeFlagBases {
    scope: u8,
    key: u8,
    value: u8,
}

const SIZES_FLAG_BASES: SizeFlagBases = SizeFlagBases {
    scope: 1 << 0,
    key: 1 << 1,
    value: 1 << 2,
};

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct Config {
    pub ident: Box<[u8]>,
    pub sizes: Sizes,
}

// endregion

// region: error types

#[derive(Debug)]
pub enum InputKind {
    Scope,
    Key,
    Value,
}

impl<'a> From<&'a str> for InputKind {
    fn from(s: &'a str) -> Self {
        match s {
            "scope" => InputKind::Scope,
            "key" => InputKind::Key,
            "value" => InputKind::Value,
            _ => unreachable!(),
        }
    }
}

error_enum! {
    #[derive(Debug)]
    pub enum Error {
        Version { existing: u32 },
        Config { existing: Config, current: Config },
        Hash { existing: Hash, calculated: Hash },
        InputLength { config_len: u32, input_len: u32, which: InputKind },
        RowType(u8),
        TooLongSize { size: u64 },
        Closed,
        AsyncFileClosed,
    } convert {
        Io => io::Error,
    }
}

pub type Result<T> = std::result::Result<T, Error>;

// endregion

// region: reader

pub struct Reader<F: Read> {
    inner: F,
    config: Config,
    hasher: Hasher,
}

impl<F: Read> Reader<F> {
    #[inline]
    pub fn config(&self) -> &Config {
        &self.config
    }

    fn read_init(inner: &mut F) -> Result<Config> {
        let version = inner.read_u32()?;
        check!(version, BS_IDENT, Error::Version { existing: version });

        let ident_len = u32_usize(inner.read_u32()?);
        let ident = inner.read_bytes(ident_len)?;

        let sizes_flag = inner.read_u8()?;
        macro_rules! skv_op_impl {
            ($($x:ident,)*) => {$(
                let $x = ((sizes_flag & SIZES_FLAG_BASES.$x) != 0).then_some(inner.read_u32()?);
            )*};
        }
        skv_op_impl!(scope, key, value,);
        let sizes = Sizes { scope, key, value };

        Ok(Config { ident, sizes })
    }

    pub fn read_row(&mut self) -> Result<Row> {
        Ok(match self.inner.read_u8()?.try_into()? {
            RowType::KV => Row::KV({
                macro_rules! skv_op_impl {
                    ($($x:ident,)*) => {$(
                        let len = u32_usize(match self.config.sizes.$x {
                            Some(len) => len,
                            None => self.inner.read_u32()?,
                        });
                        let $x = self.inner.read_bytes(len)?;
                        self.hasher.update(&$x);
                    )*};
                }
                skv_op_impl!(scope, key, value,);
                KV { scope, key, value }
            }),
            RowType::Hash => Row::Hash({
                let existing = self.inner.read_hash()?;
                let calculated = *self.hasher.finalize().as_bytes();
                check!(existing, calculated, Error::Hash { existing, calculated });
                self.hasher.reset();
                calculated
            }),
            RowType::End => Row::End,
        })
    }

    pub fn init(mut inner: F) -> Result<Reader<F>> {
        let config = Reader::read_init(&mut inner)?;
        Ok(Reader { inner, config, hasher: Hasher::new() })
    }
}

impl<F: Read> Iterator for Reader<F> {
    type Item = Result<Row>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.read_row() {
            Ok(Row::End) => None,
            result => Some(result),
        }
    }
}

// endregion

// region: writer

pub struct Writer<F: Write> {
    inner: F,
    config: Config,
    hasher: Hasher,
    closed: bool,
}

impl<F: Write> Writer<F> {
    #[inline]
    pub fn config(&self) -> &Config {
        &self.config
    }

    #[inline]
    fn close_guard(&self) -> Result<()> {
        check!(self.closed, false, Error::Closed);
        Ok(())
    }

    fn write_init(&mut self) -> Result<()> {
        self.inner.write_u32(BS_IDENT)?;

        self.inner.write_u32(usize_u32(self.config.ident.len())?)?;
        self.inner.write_bytes(self.config.ident.clone())?;

        self.inner.write_u8(self.config.sizes.flag())?;
        macro_rules! skv_op_impl {
            ($($x:ident,)*) => {$(
                self.inner.write_u32(self.config.sizes.$x.unwrap_or(0))?;
            )*};
        }
        skv_op_impl!(scope, key, value,);

        // self.inner.flush()?;
        Ok(())
    }

    pub fn write_kv(&mut self, kv: KV) -> Result<()> {
        self.close_guard()?;
        
        self.inner.write_u8(RowType::KV as u8)?;

        macro_rules! skv_op_impl {
            ($($x:ident,)*) => {$({
                let input_len = usize_u32(kv.$x.len())?;
                match self.config.sizes.$x {
                    Some(config_len) => {
                        check!(config_len, input_len, Error::InputLength {
                            config_len,
                            input_len,
                            which: stringify!($x).into(),
                        })
                    },
                    None => self.inner.write_u32(input_len)?,
                }
                self.hasher.update(&kv.$x);
                self.inner.write_bytes(kv.$x)?;
            })*};
        }
        skv_op_impl!(scope, key, value,);

        // self.inner.flush()?;
        Ok(())
    }

    pub fn write_hash(&mut self) -> Result<Hash> {
        self.close_guard()?;

        self.inner.write_u8(RowType::Hash as u8)?;

        let hash = *self.hasher.finalize().as_bytes();
        self.inner.write_bytes(hash)?;

        // self.inner.flush()?;
        Ok(hash)
    }

    fn write_end(&mut self) -> Result<()> {
        self.inner.write_u8(RowType::End as u8)?;

        // self.inner.flush()?;
        Ok(())
    }

    pub fn close(&mut self) -> Result<()> {
        self.close_guard()?;
        self.write_hash()?;
        self.write_end()?;
        self.closed = true;
        Ok(())
    }

    pub fn init(inner: F, config: Config) -> Result<Writer<F>> {
        let mut _self = Writer { inner, config, hasher: Hasher::new(), closed: false };
        _self.write_init()?;
        Ok(_self)
    }
}

impl Writer<std::fs::File> {
    #[inline]
    pub fn fsync(&mut self) -> Result<()> {
        Ok(self.inner.sync_all()?)
    }

    pub fn datasync(&mut self) -> Result<()> {
        Ok(self.inner.sync_data()?)
    }

    pub fn close_file(&mut self) -> Result<()> {
        self.close()?;
        self.fsync()?;
        Ok(())
    }
}

impl<F: Write> Drop for Writer<F> {
    fn drop(&mut self) {
        if !self.closed {
            self.close().expect("FATAL: Error occurred during closing");
        }
    }
}

// endregion
