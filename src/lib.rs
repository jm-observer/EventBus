#![feature(associated_type_defaults)]
// #![feature(trait_upcasting)]

pub mod sub_bus;
pub mod worker;

use crate::sub_bus::{CopyOfSubBus, SubBus};
use log::{debug, error};
use std::any::{Any, TypeId};
use std::collections::HashMap;

use std::sync::Arc;

use tokio::spawn;
use tokio::sync::mpsc::error::SendError;
use tokio::sync::mpsc::unbounded_channel;
use tokio::sync::{
    mpsc::{UnboundedReceiver, UnboundedSender},
    oneshot,
};

use crate::worker::IdentityOfTx;
use worker::{CopyOfWorker, IdentityOfRx, WorkerId};

pub type Event = Arc<dyn Any + Send + Sync + 'static>;

#[derive(Debug)]
pub enum BusError {
    ChannelErr,
}

impl<T> From<SendError<T>> for BusError {
    fn from(_: SendError<T>) -> Self {
        Self::ChannelErr
    }
}
impl From<oneshot::error::RecvError> for BusError {
    fn from(_: oneshot::error::RecvError) -> Self {
        Self::ChannelErr
    }
}

pub enum BusData {
    Login(oneshot::Sender<(IdentityOfRx, IdentityOfTx)>),
    Subscribe(WorkerId, TypeId),
    DispatchEvent(WorkerId, Event),
    Drop(WorkerId),
}

#[derive(Clone)]
pub struct CopyOfBus {
    tx: UnboundedSender<BusData>,
}

impl CopyOfBus {
    pub async fn login(&self) -> Result<(IdentityOfRx, IdentityOfTx), BusError> {
        let (tx, rx) = oneshot::channel();
        self.tx.send(BusData::Login(tx))?;
        Ok(rx.await?)
    }
    //
    // pub async fn send_event(&self, event: Event) -> Result<()> {
    //     self.tx
    //         .send(BusData::DispatchEvent(event))
    //         .await
    //         .map_err(|_| anyhow!("fail to contact bus"))
    // }
}

pub struct Bus {
    rx: UnboundedReceiver<BusData>,
    tx: UnboundedSender<BusData>,
    workers: HashMap<WorkerId, CopyOfWorker>,
    sub_buses: HashMap<TypeId, CopyOfSubBus>,
}

impl Bus {
    pub fn init() -> CopyOfBus {
        let (tx, rx) = unbounded_channel();
        Self {
            rx,
            tx: tx.clone(),
            workers: Default::default(),
            sub_buses: Default::default(),
        }
        .run();
        CopyOfBus { tx }
    }
    fn run(mut self) {
        spawn(async move {
            while let Some(event) = self.rx.recv().await {
                match event {
                    BusData::Login(tx) => {
                        let (identity_rx, identify_tx, copy_of_worker) = self.init_worker();
                        self.workers.insert(copy_of_worker.id(), copy_of_worker);
                        if tx.send((identity_rx, identify_tx)).is_err() {
                            error!("login fail: tx ack fail");
                        }
                    }
                    BusData::Drop(worker_id) => {
                        debug!("{:?} Drop", worker_id);
                        if let Some(worker) = self.workers.remove(&worker_id) {
                            for ty_id in worker.subscribe_events() {
                                let should_remove =
                                    if let Some(sub_bus) = self.sub_buses.get_mut(&ty_id) {
                                        sub_bus.send_unsubscribe(worker_id).await == 0
                                    } else {
                                        //todo
                                        false
                                    };
                                if should_remove {
                                    self.sub_buses.remove(ty_id);
                                }
                            }
                        } else {
                            // todo
                        }
                    }
                    BusData::DispatchEvent(worker_id, event) => {
                        debug!(
                            "{:?} DispatchEvent {:?}",
                            worker_id,
                            event.as_ref().type_id()
                        );
                        if let Some(sub_buses) = self.sub_buses.get(&event.as_ref().type_id()) {
                            sub_buses.send_event(event).await;
                        }
                    }
                    BusData::Subscribe(worker_id, typeid) => {
                        debug!("Subscribe {:?} {:?}", worker_id, typeid);
                        if let Some(worker) = self.workers.get_mut(&worker_id) {
                            worker.subscribe_event(typeid);
                            if let Some(sub_buses) = self.sub_buses.get_mut(&typeid) {
                                sub_buses.send_subscribe(worker.init_subscriber()).await;
                            } else {
                                let mut copy = SubBus::init(typeid);
                                copy.send_subscribe(worker.init_subscriber()).await;
                                self.sub_buses.insert(typeid, copy);
                            }
                        }
                    }
                }
            }
        });
    }

    fn init_worker(&self) -> (IdentityOfRx, IdentityOfTx, CopyOfWorker) {
        let (tx_event, rx_event) = unbounded_channel();
        let id = WorkerId::default();
        (
            IdentityOfRx::init(id, rx_event, self.tx.clone()),
            IdentityOfTx::init(id, self.tx.clone()),
            CopyOfWorker::init(id, tx_event),
        )
    }
}
