#![feature(trait_upcasting)]

mod event_bus;

use std::any::{Any, TypeId};
use std::collections::{HashMap, HashSet};
use std::fmt::Debug;
use std::sync::Arc;
use std::time::Duration;
use tokio::spawn;
use tokio::sync::{mpsc::{Receiver, Sender}, mpsc, oneshot, RwLock};
use tokio::time::sleep;
use crate::event_bus::CopyOfSubBus;

pub type Event = dyn Any + Send + Sync + 'static;

pub struct BusInterface {
    tx: Sender<Arc<Event>>,
    tx_subscribe: Sender<(TypeId, WorkerId)>,
}

pub enum BusData {
    Login(oneshot::Sender<IdentityOfWorker>),
    Logout(WorkerId),
    DispatchEvent(TypeId, WorkerId),

}
pub struct IdentityOfWorker {
    id: WorkerId,
    rx: Receiver<Arc<Event>>,
}
struct CopyOfWorker {
    id: WorkerId,
    tx: Sender<Arc<Event>>,
    subscribe_events: Vec<TypeId>,
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
    pub fn run(mut self) {
        spawn(async move {
            while let Some(event) = self.rx.recv().await {
                // let type_id = event.as_ref().type_id();
                // println!("Bus recv {:?}", type_id);
                // if let Some(workers) = self.workers_subscribe.get(&type_id) {
                //     for x in workers.iter() {
                //         if let Some(worker) = self.workers.get(x) {
                //             if let Err(_) = worker.send(event.clone()).await {
                //                 println!("error");
                //             }
                //         }
                //     }
                // }
            }
        });
    }
    // pub fn init() -> Self {
    //     let (tx, rx) =mpsc::channel(100);
    //     Self {
    //         workers: Default::default(),
    //         rx,
    //         tx,
    //         sub_buses: Default::default(),
    //     }
    // }
    // pub fn interface(&self) -> BusInterface {
    //     BusInterface {
    //         tx: self.tx.clone(),
    //         tx_subscribe: self.tx_subscribe.clone(),
    //     }
    // }
    // pub fn register(&mut self, worker_id: WorkerId, tx: Sender<Arc<Event>>)  {
    //     self.workers.insert(worker_id, tx);
    // }
    // pub fn subscribe(&mut self, worker_id: WorkerId, type_id: TypeId)  {
    //     println!("subscribe {:?}", type_id);
    //     let Some(workers) = self.workers_subscribe.get_mut(&type_id) else {
    //         let mut workers = Vec::new();
    //         workers.push(worker_id);
    //         self.workers_subscribe.insert(type_id, workers);
    //         return;
    //     };
    //     workers.push(worker_id);
    // }
}

#[derive(Debug, Eq, PartialEq, Copy, Clone, Hash)]
pub struct WorkerId(usize);