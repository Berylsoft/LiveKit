use tokio::sync::{
    oneshot::{channel as one_channel, Sender as OneTx},
    mpsc::{unbounded_channel as _req_channel, UnboundedSender as _ReqTx},
};
use crate::*;

pub(crate) type ReqPayload = (Request, OneTx<Result<()>>);

pub struct ReqTx {
    // access inner is generally safe
    pub(crate) inner: _ReqTx<ReqPayload>,
}

impl ReqTx {
    pub(crate) async fn request(&self, req: Request) -> std::result::Result<Result<()>, Option<Request>> {
        let (res_tx, res_rx) = one_channel::<Result<()>>();
        self.inner.send((req, res_tx)).map_err(|payload| Some(payload.0.0))?;
        res_rx.await.map_err(|_| None)
    }
}

impl Clone for ReqTx {
    fn clone(&self) -> Self {
        ReqTx { inner: self.inner.clone() }
    }
}

pub struct CloseHandle {
    inner: ReqTx,
}

impl CloseHandle {
    pub async fn close_and_wait(self) -> Result<()> {
        self.inner.request(Request::Close).await.unwrap_or(Err(Error::AsyncFileClosed))
    }
}

pub(crate) async fn spawn(path: PathBuf) -> Result<(ReqTx, CloseHandle)> {
    let (req_tx, mut req_rx) = _req_channel::<ReqPayload>();
    let req_tx = ReqTx { inner: req_tx };
    let rt = tokio::runtime::Handle::current();
    let mut ctx = rt.spawn_blocking(move || {
        WriterContext::init(&path)
    }).await.map_err(|err| std::io::Error::new(
        std::io::ErrorKind::Other,
        err,
    ))??;
    rt.spawn_blocking(move || {
        loop {
            if let Some((req, res_tx)) = req_rx.blocking_recv() {
                res_tx.send(ctx.exec(req)).expect("FATAL: all request sender dropped");
            } else {
                ctx.exec(Request::Close).expect("FATAL: Error occurred during closing");
                break;
            }
        }
    });
    Ok((req_tx.clone(), CloseHandle { inner: req_tx }))
}
