use tokio::sync::{
    oneshot::{channel as one_channel, Sender as OneTx, Receiver as OneRx},
    mpsc::{unbounded_channel as _req_channel, UnboundedSender as _ReqTx},
};

use std::fmt::Debug;

pub trait Executor: 'static + Sized + Send {
    type Req: Send + Debug;
    type Res: Send + Debug;

    fn exec(&mut self, req: Self::Req) -> Self::Res;
    fn close(self) {}
}

pub type ReqPayload<E> = (<E as Executor>::Req, OneTx<<E as Executor>::Res>);

pub struct ReqTx<E: Executor> {
    // access inner is generally safe
    pub inner: _ReqTx<ReqPayload<E>>,
}

impl<E: Executor> ReqTx<E> {
    pub async fn request(&self, req: E::Req) -> Result<E::Res, Option<E::Req>> {
        let (res_tx, res_rx) = one_channel::<E::Res>();
        self.inner.send((req, res_tx)).map_err(|payload| Some(payload.0.0))?;
        res_rx.await.map_err(|_| None)
    }
}

impl<E: Executor> Clone for ReqTx<E> {
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

pub fn spawn<E: Executor>(mut ctx: E) -> (ReqTx<E>, CloseHandle) {
    let (tx, mut wait) = one_channel();
    let (finish, rx) = one_channel();
    let (req_tx, mut req_rx) = _req_channel::<ReqPayload<E>>();
    tokio::spawn(async move {
        loop {
            tokio::select! {
                biased;
                Ok(()) = &mut wait => {
                    ctx.close();
                    finish.send(()).unwrap();
                    break;
                }
                maybe_req = req_rx.recv() => {
                    if let Some((req, res_tx)) = maybe_req {
                        res_tx.send(ctx.exec(req)).unwrap();
                    } else {
                        ctx.close();
                        finish.send(()).unwrap();
                        break;
                    }
                }
            }
        }    
    });
    (ReqTx { inner: req_tx }, CloseHandle { tx: Some(tx), rx })
}
