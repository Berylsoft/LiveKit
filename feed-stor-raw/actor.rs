use tokio::sync::{
    oneshot::{channel as one_channel, Sender as OneTx, Receiver as OneRx},
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
    tx: Option<OneTx<()>>,
    rx: OneRx<()>,
}

impl CloseHandle {
    pub fn close(&mut self) -> Option<()> {
        self.tx.take().unwrap().send(()).ok()
    }

    pub async fn wait(self) -> Option<()> {
        self.rx.await.ok()
    }
}

pub(crate) fn spawn(mut ctx: WriterContext) -> (ReqTx, CloseHandle) {
    let (tx, mut wait) = one_channel();
    let (finish, rx) = one_channel();
    let (req_tx, mut req_rx) = _req_channel::<ReqPayload>();
    tokio::spawn(async move {
        loop {
            tokio::select! {
                biased;
                Ok(()) = &mut wait => {
                    ctx.close().await;
                    finish.send(()).unwrap();
                    break;
                }
                maybe_req = req_rx.recv() => {
                    if let Some((req, res_tx)) = maybe_req {
                        res_tx.send(ctx.exec(req).await).unwrap();
                    } else {
                        ctx.close().await;
                        finish.send(()).unwrap();
                        break;
                    }
                }
            }
        }    
    });
    (ReqTx { inner: req_tx }, CloseHandle { tx: Some(tx), rx })
}
