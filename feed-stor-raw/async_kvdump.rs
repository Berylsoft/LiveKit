use tokio::io::{AsyncWrite, AsyncWriteExt};
use kvdump::*;

pub struct AsyncWriter<F: AsyncWrite + Unpin> {
    inner: F,
    config: Config,
    hasher: Hasher,
    closed: bool,
}

impl<F: AsyncWrite + Unpin> AsyncWriter<F> {
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

        self.inner.flush().await?;
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

        self.inner.flush().await?;
        Ok(())
    }

    pub async fn write_hash(&mut self) -> Result<Hash> {
        self.close_guard()?;

        self.inner.write_u8(RowType::Hash as u8).await?;

        let hash = *self.hasher.finalize().as_bytes();
        self.inner.write_all(&hash).await?;

        self.inner.flush().await?;
        Ok(hash)
    }

    async fn write_end(&mut self) -> Result<()> {
        self.inner.write_u8(RowType::End as u8).await?;

        self.inner.flush().await?;
        Ok(())
    }

    pub async fn close(&mut self) -> Result<()> {
        self.close_guard()?;
        self.write_hash().await?;
        self.write_end().await?;
        self.closed = true;
        Ok(())
    }

    pub async fn init(inner: F, config: Config) -> Result<AsyncWriter<F>> {
        let mut _self = AsyncWriter { inner, config, hasher: Hasher::new(), closed: false };
        _self.write_init().await?;
        Ok(_self)
    }
}

impl AsyncWriter<tokio::fs::File> {
    #[inline]
    pub async fn fsync(&mut self) -> Result<()> {
        Ok(self.inner.sync_all().await?)
    }

    #[inline]
    pub async fn datasync(&mut self) -> Result<()> {
        Ok(self.inner.sync_data().await?)
    }

    pub async fn close_file(&mut self) -> Result<()> {
        self.close().await?;
        self.fsync().await?;
        Ok(())
    }
}

impl<F: AsyncWrite + Unpin> Drop for AsyncWriter<F> {
    fn drop(&mut self) {
        if !self.closed {
            tokio::runtime::Handle::try_current().unwrap().block_on(self.close()).expect("FATAL: Error occurred during closing");
        }
    }
}
