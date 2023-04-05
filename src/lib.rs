#![feature(trait_upcasting)]

mod sub_bus;
mod worker;

use crate::sub_bus::{CopyOfSubBus, SubBus, SubBusData};
use log::error;
use std::any::{Any, TypeId};
use std::collections::{HashMap, HashSet};
use std::fmt::Debug;
use std::sync::Arc;
use std::time::Duration;
use tokio::spawn;
use tokio::sync::mpsc::channel;
use tokio::sync::{
    mpsc,
    mpsc::{Receiver, Sender},
    oneshot, RwLock,
};
use tokio::time::sleep;
use worker::{CopyOfWorker, IdentityOfWorker, WorkerId};

pub type Event = Arc<dyn Any + Send + Sync + 'static>;

pub enum BusData {
    Login(oneshot::Sender<IdentityOfWorker>),
    Logout(WorkerId),
    Subscribe(WorkerId, TypeId),
    DispatchEvent(Event),
}

pub struct CopyOfBus {
    tx: Sender<BusData>,
}

pub struct Bus {
    rx: Receiver<BusData>,
    tx: Sender<BusData>,
    workers: HashMap<WorkerId, CopyOfWorker>,
    sub_buses: HashMap<TypeId, CopyOfSubBus>,
}

impl Bus {
    pub fn init() -> CopyOfBus {
        let (tx, rx) = channel(1024);
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
                        let (identity_worker, copy_of_worker) = self.init_worker();
                        self.workers.insert(copy_of_worker.id(), copy_of_worker);
                        if tx.send(identity_worker).is_err() {
                            error!("login fail: tx ack fail");
                        }
                    }
                    BusData::Logout(worker_id) => {
                        if let Some(worker) = self.workers.remove(&worker_id) {
                            for ty_id in worker.subscribe_events() {
                                if let Some(sub_bus) = self.sub_buses.get(&ty_id) {
                                    if sub_bus
                                        .tx
                                        .send(SubBusData::Unsubscribe(worker_id))
                                        .await
                                        .is_err()
                                    {
                                        // todo
                                    }
                                } else {
                                    //todo
                                }
                            }
                        } else {
                            // todo
                        }
                    }
                    BusData::DispatchEvent(event) => {
                        if let Some(sub_buses) = self.sub_buses.get(&event.type_id()) {
                            sub_buses.send_event(event).await;
                        }
                    }
                    BusData::Subscribe(worker_id, typeid) => {
                        if let Some(worker) = self.workers.get_mut(&worker_id) {
                            worker.subscribe_event(typeid);
                            if let Some(sub_buses) = self.sub_buses.get(&typeid) {
                                sub_buses.send_subscribe(worker_id, worker.tx()).await;
                            } else {
                                let copy = SubBus::init(typeid);
                                copy.send_subscribe(worker_id, worker.tx()).await;
                                self.sub_buses.insert(typeid, copy);
                            }
                        }
                    }
                }
            }
        });
    }

    fn init_worker(&self) -> (IdentityOfWorker, CopyOfWorker) {
        let (tx_event, rx_event) = channel(1024);
        let id = WorkerId::default();
        (
            IdentityOfWorker::init(id, rx_event, self.tx.clone()),
            CopyOfWorker::init(id, tx_event),
        )
    }
}
