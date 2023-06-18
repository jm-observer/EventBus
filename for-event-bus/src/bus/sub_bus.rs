use crate::bus::BusEvent;
use log::{debug, error};
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
}
#[allow(dead_code)]
pub(crate) struct EntryOfSubBus {
    type_id: TypeId,
    subscribers: HashSet<WorkerId>,
    tx: Sender<SubBusData>,
}

// impl Drop for EntryOfSubBus {
//     fn drop(&mut self) {
//         if self.tx.try_send(SubBusData::Drop).is_err() {
//             error!("{:?} send SubBusData::Drop fail", self.type_id);
//         }
//     }
// }

impl EntryOfSubBus {
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
    rx: Receiver<SubBusData>,
    subscribers: HashMap<WorkerId, Worker>,
}

impl<const CAP: usize> SubBus<CAP> {
    pub(crate) fn init(type_id: TypeId) -> EntryOfSubBus {
        let (tx, rx) = channel(CAP);
        Self {
            type_id,
            rx,
            subscribers: Default::default(),
        }
        .run();
        EntryOfSubBus {
            type_id,
            subscribers: Default::default(),
            tx,
        }
    }
    fn run(mut self) {
        spawn(async move {
            while let Some(data) = self.rx.recv().await {
                match data {
                    SubBusData::Subscribe(subscriber) => {
                        self.subscribers.insert(subscriber.id(), subscriber);
                    }
                    SubBusData::Unsubscribe(worker_id) => {
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
                }
            }
            debug!("{:?} end", self.type_id);
        });
    }
}
