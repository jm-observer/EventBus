use crate::worker::{Subscriber, WorkerId};
use crate::Event;
use log::{debug, error};
use std::any::TypeId;
use std::collections::{HashMap, HashSet};

use tokio::spawn;
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};

pub(crate) enum SubBusData {
    Subscribe(Subscriber),
    Unsubscribe(WorkerId),
    Event(Event),
    Drop,
}
#[allow(dead_code)]
pub(crate) struct CopyOfSubBus {
    type_id: TypeId,
    subscribers: HashSet<WorkerId>,
    tx: UnboundedSender<SubBusData>,
}

impl Drop for CopyOfSubBus {
    fn drop(&mut self) {
        if self.tx.send(SubBusData::Drop).is_err() {
            error!("{:?} send SubBusData::Drop fail", self.type_id);
        }
    }
}

impl CopyOfSubBus {
    pub async fn send_event(&self, event: Event) {
        if self.tx.send(SubBusData::Event(event)).is_err() {
            error!("fail to send event to sub bus");
        }
    }

    pub async fn send_subscribe(&mut self, subscriber: Subscriber) {
        self.subscribers.insert(subscriber.id());
        if self.tx.send(SubBusData::Subscribe(subscriber)).is_err() {
            error!("fail to send subscribe to sub bus");
        }
    }
    pub async fn send_unsubscribe(&mut self, worker_id: WorkerId) -> usize {
        self.subscribers.remove(&worker_id);
        if self.tx.send(SubBusData::Unsubscribe(worker_id)).is_err() {
            error!("fail to send subscribe to sub bus");
        }
        self.subscribers.len()
    }
}
#[allow(dead_code)]
/// 子事件总线
pub struct SubBus {
    type_id: TypeId,
    rx: UnboundedReceiver<SubBusData>,
    subscribers: HashMap<WorkerId, Subscriber>,
}

impl SubBus {
    pub(crate) fn init(type_id: TypeId) -> CopyOfSubBus {
        let (tx, rx) = unbounded_channel();
        Self {
            type_id,
            rx,
            subscribers: Default::default(),
        }
        .run();
        CopyOfSubBus {
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
