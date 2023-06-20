use crate::bus::BusEvent;
use log::{debug, error, trace};
use std::any::TypeId;
use std::collections::{HashMap, HashSet};

use crate::worker::{Worker, WorkerId};
use tokio::spawn;
use tokio::sync::mpsc::{channel, Receiver, Sender};

pub(crate) enum SubBusData {
    Subscribe(Worker),
    Unsubscribe(WorkerId),
    Event(BusEvent),
    Drop,
    Trace,
}
#[allow(dead_code)]
pub(crate) struct EntryOfSubBus {
    type_id: TypeId,
    name: &'static str,
    subscribers: HashSet<WorkerId>,
    tx: Sender<SubBusData>,
}

impl EntryOfSubBus {
    pub fn name(&self) -> &'static str {
        self.name.clone()
    }

    pub async fn send_trace(&self) {
        if self.tx.send(SubBusData::Trace).await.is_err() {
            error!("fail to send event to sub bus");
        }
    }

    pub async fn send_event(&self, event: BusEvent) {
        if self.tx.send(SubBusData::Event(event)).await.is_err() {
            error!("fail to send event to sub bus");
        }
    }

    pub async fn send_subscribe(&mut self, subscriber: Worker) {
        self.subscribers.insert(subscriber.id());
        if self
            .tx
            .send(SubBusData::Subscribe(subscriber))
            .await
            .is_err()
        {
            error!("fail to send subscribe to sub bus");
        }
    }
    pub async fn send_unsubscribe(&mut self, worker_id: WorkerId) -> usize {
        self.subscribers.remove(&worker_id);
        if self
            .tx
            .send(SubBusData::Unsubscribe(worker_id))
            .await
            .is_err()
        {
            error!("fail to send subscribe to sub bus");
        }
        self.subscribers.len()
    }

    pub async fn send_drop(&self) {
        if self.tx.send(SubBusData::Drop).await.is_err() {
            error!("fail to send drop to sub bus");
        }
    }
}
#[allow(dead_code)]
/// 子事件总线
pub struct SubBus<const CAP: usize> {
    type_id: TypeId,
    name: &'static str,
    rx: Receiver<SubBusData>,
    subscribers: HashMap<WorkerId, Worker>,
}

impl<const CAP: usize> SubBus<CAP> {
    pub(crate) fn init(type_id: TypeId, name: &'static str) -> EntryOfSubBus {
        let (tx, rx) = channel(CAP);
        Self {
            type_id,
            name: name.clone(),
            rx,
            subscribers: Default::default(),
        }
        .run();
        EntryOfSubBus {
            type_id,
            name,
            subscribers: Default::default(),
            tx,
        }
    }
    fn run(mut self) {
        spawn(async move {
            while let Some(data) = self.rx.recv().await {
                match data {
                    SubBusData::Subscribe(subscriber) => {
                        trace!("worker {} subscribe {}", subscriber.id(), self.name);
                        self.subscribers.insert(subscriber.id(), subscriber);
                    }
                    SubBusData::Unsubscribe(worker_id) => {
                        trace!("worker {} unsubscribe {}", worker_id, self.name);
                        self.subscribers.remove(&worker_id);
                    }
                    SubBusData::Event(event) => {
                        for subscriber in self.subscribers.values() {
                            subscriber.send(event.clone()).await
                        }
                    }
                    SubBusData::Drop => {
                        break;
                    }
                    SubBusData::Trace => {
                        debug!("subscriber of {}: ", self.name);
                        for subscriber in self.subscribers.values() {
                            debug!("\t{}", subscriber.id())
                        }
                    }
                }
            }
            debug!("sub-bus {} drop", self.name);
        });
    }
}
