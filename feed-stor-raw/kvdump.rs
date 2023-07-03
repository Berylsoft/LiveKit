pub const BS_IDENT: u32 = 0x42650000;

use tokio::io::{self, AsyncWrite, AsyncWriteExt};
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

// region: writer

pub struct Writer<F: AsyncWrite + Unpin> {
    inner: F,
    config: Config,
    hasher: Hasher,
    closed: bool,
}

impl<F: AsyncWrite + Unpin> Writer<F> {
    #[inline]
    pub fn config(&self) -> &Config {
        &self.config
    }

    #[inline]
    fn close_guard(&self) -> Result<()> {
        check!(self.closed, false, Error::Closed);
        Ok(())
    }

    async fn write_init(&mut self) -> Result<()> {
        self.inner.write_u32(BS_IDENT).await?;

        self.inner.write_u32(usize_u32(self.config.ident.len())?).await?;
        self.inner.write_all(&self.config.ident).await?;

        self.inner.write_u8(self.config.sizes.flag()).await?;
        macro_rules! skv_op_impl {
            ($($x:ident,)*) => {$(
                self.inner.write_u32(self.config.sizes.$x.unwrap_or(0)).await?;
            )*};
        }
        skv_op_impl!(scope, key, value,);

        // self.inner.flush()?;
        Ok(())
    }

    pub async fn write_kv(&mut self, kv: KV) -> Result<()> {
        self.close_guard()?;
        
        self.inner.write_u8(RowType::KV as u8).await?;

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
                    None => self.inner.write_u32(input_len).await?,
                }
                self.hasher.update(&kv.$x);
                self.inner.write_all(&kv.$x).await?;
            })*};
        }
        skv_op_impl!(scope, key, value,);

        // self.inner.flush()?;
        Ok(())
    }

    pub async fn write_hash(&mut self) -> Result<Hash> {
        self.close_guard()?;

        self.inner.write_u8(RowType::Hash as u8).await?;

        let hash = *self.hasher.finalize().as_bytes();
        self.inner.write_all(&hash).await?;

        // self.inner.flush()?;
        Ok(hash)
    }

    async fn write_end(&mut self) -> Result<()> {
        self.inner.write_u8(RowType::End as u8).await?;

        // self.inner.flush()?;
        Ok(())
    }

    pub async fn close(&mut self) -> Result<()> {
        self.close_guard()?;
        self.write_hash().await?;
        self.write_end().await?;
        self.closed = true;
        Ok(())
    }

    pub async fn init(inner: F, config: Config) -> Result<Writer<F>> {
        let mut _self = Writer { inner, config, hasher: Hasher::new(), closed: false };
        _self.write_init().await?;
        Ok(_self)
    }
}

impl Writer<tokio::fs::File> {
    #[inline]
    pub async fn fsync(&mut self) -> Result<()> {
        Ok(self.inner.sync_all().await?)
    }

    pub async fn datasync(&mut self) -> Result<()> {
        Ok(self.inner.sync_data().await?)
    }

    pub async fn close_file(&mut self) -> Result<()> {
        self.close().await?;
        self.fsync().await?;
        Ok(())
    }
}

impl<F: AsyncWrite + Unpin> Drop for Writer<F> {
    fn drop(&mut self) {
        if !self.closed {
            tokio::runtime::Handle::try_current().unwrap().block_on(self.close()).expect("FATAL: Error occurred during closing");
        }
    }
}

// endregion
