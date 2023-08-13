use crate::bus::sub_bus::{EntryOfSubBus, SubBus};
use crate::worker::identity::{IdentityCommon, IdentityOfRx, IdentityOfSimple, Merge};
use crate::worker::{CopyOfWorker, ToWorker, WorkerId};
use crate::{Event, IdentityOfMerge};
use log::{debug, error};
use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::spawn;
use tokio::sync::mpsc::error::SendError;
use tokio::sync::mpsc::{channel, Receiver, Sender};
use tokio::sync::oneshot;
use tokio::time::sleep;

mod sub_bus;

pub type BusEvent = Arc<dyn Event>;

#[derive(Debug)]
pub enum BusError {
    ChannelErr,
    DowncastErr,
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
    Login(oneshot::Sender<IdentityCommon>, String),
    // SimpleLogin(oneshot::Sender<IdentityCommon>, String),
    Subscribe(WorkerId, TypeId, &'static str),
    DispatchEvent(WorkerId, BusEvent),
    Drop(WorkerId),
    Trace,
}

#[derive(Clone)]
pub struct EntryOfBus {
    tx: Sender<BusData>,
}

impl EntryOfBus {
    pub async fn login<W: ToWorker>(&self) -> Result<IdentityOfRx, BusError> {
        let (tx, rx) = oneshot::channel();
        self.tx.send(BusData::Login(tx, W::name())).await?;
        Ok(rx.await?.into())
    }
    pub async fn simple_login<W: ToWorker, T: Event>(
        &self,
    ) -> Result<IdentityOfSimple<T>, BusError> {
        let (tx, rx) = oneshot::channel();
        self.tx.send(BusData::Login(tx, W::name())).await?;
        let rx: IdentityOfSimple<T> = rx.await?.into();
        rx.subscribe().await?;
        Ok(rx)
    }
    pub async fn merge_login<W: ToWorker, T: Event + Merge>(
        &self,
    ) -> Result<IdentityOfMerge<T>, BusError> {
        let (tx, rx) = oneshot::channel();
        self.tx.send(BusData::Login(tx, W::name())).await?;
        let rx: IdentityOfMerge<T> = rx.await?.into();
        rx.subscribe().await?;
        Ok(rx)
    }
}

pub struct Bus<const CAP: usize> {
    rx: Receiver<BusData>,
    tx: Sender<BusData>,
    workers: HashMap<WorkerId, CopyOfWorker>,
    sub_buses: HashMap<TypeId, EntryOfSubBus>,
}

impl<const CAP: usize> Drop for Bus<CAP> {
    fn drop(&mut self) {
        debug!("bus drop");
    }
}

impl<const CAP: usize> Bus<CAP> {
    pub fn init() -> EntryOfBus {
        let (tx, rx) = channel(CAP);
        Self {
            rx,
            tx: tx.clone(),
            workers: Default::default(),
            sub_buses: Default::default(),
        }
        .run();
        EntryOfBus { tx }
    }
    fn run(mut self) {
        spawn(async move {
            let tx = self.tx.clone();
            spawn(async move {
                let time = Duration::from_secs(30);
                loop {
                    sleep(time).await;
                    if tx.send(BusData::Trace).await.is_err() {
                        return;
                    }
                }
            });
            while let Some(event) = self.rx.recv().await {
                match event {
                    BusData::Login(tx, name) => {
                        let (identity_rx, copy_of_worker) = self.init_worker(name);
                        self.workers.insert(copy_of_worker.id(), copy_of_worker);
                        if tx.send(identity_rx).is_err() {
                            error!("login fail: tx ack fail");
                        }
                    }
                    BusData::Drop(worker_id) => {
                        debug!("{} Drop", worker_id);
                        if let Some(worker) = self.workers.remove(&worker_id) {
                            for ty_id in worker.subscribe_events() {
                                let should_remove =
                                    if let Some(sub_bus) = self.sub_buses.get_mut(&ty_id) {
                                        sub_bus.send_unsubscribe(worker_id.clone()).await == 0
                                    } else {
                                        //todo
                                        false
                                    };
                                if should_remove {
                                    if let Some(sub_bus) = self.sub_buses.remove(ty_id) {
                                        sub_bus.send_drop().await;
                                    }
                                }
                            }
                        } else {
                            // todo
                        }
                    }
                    BusData::DispatchEvent(worker_id, event) => {
                        if let Some(sub_buses) = self.sub_buses.get(&event.as_ref().type_id()) {
                            debug!("{} dispatch {}", worker_id, sub_buses.name());
                            sub_buses.send_event(event).await;
                        } else {
                            debug!("{} dispatch type_id {:?} that no one subscribe", worker_id, event.type_id());
                        }
                    }
                    BusData::Subscribe(worker_id, typeid, name) => {
                        debug!("{} subscribe {}", worker_id, name);
                        if let Some(worker) = self.workers.get_mut(&worker_id) {
                            worker.subscribe_event(typeid);
                            if let Some(sub_buses) = self.sub_buses.get_mut(&typeid) {
                                sub_buses.send_subscribe(worker.init_subscriber()).await;
                            } else {
                                let mut copy = SubBus::<CAP>::init(typeid, name);
                                copy.send_subscribe(worker.init_subscriber()).await;
                                self.sub_buses.insert(typeid, copy);
                            }
                        }
                    }
                    BusData::Trace => {
                        for (_, sub_bus) in self.sub_buses.iter() {
                            sub_bus.send_trace().await;
                        }
                    }
                }
            }
        });
    }

    // fn init_worker(&self) -> (IdentityOfRx, CopyOfWorker) {
    //     let (tx_event, rx_event) = unbounded_channel();
    //     let id = WorkerId::default();
    //     (
    //         IdentityOfRx::init(id, rx_event, self.tx.clone()),
    //         CopyOfWorker::init(id, tx_event),
    //     )
    // }
    fn init_worker(&self, name: String) -> (IdentityCommon, CopyOfWorker) {
        let (tx_event, rx_event) = channel(CAP);
        let id = WorkerId::init(name);
        (
            IdentityCommon {
                id: id.clone(),
                rx_event,
                tx_data: self.tx.clone(),
            },
            CopyOfWorker::init(id, tx_event),
        )
    }
}
