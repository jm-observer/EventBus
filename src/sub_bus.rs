use crate::worker::{Subscriber, WorkerId};
use crate::Event;
use log::error;
use std::any::TypeId;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::spawn;
use tokio::sync::mpsc::{channel, Receiver, Sender};

pub enum SubBusData {
    Subscribe(Subscriber),
    Unsubscribe(WorkerId),
    Event(Event),
}

pub struct CopyOfSubBus {
    type_id: TypeId,
    subscriber_num: usize,
    pub(crate) tx: Sender<SubBusData>,
}

impl CopyOfSubBus {
    pub async fn send_event(&self, event: Event) {
        if self.tx.send(SubBusData::Event(event)).await.is_err() {
            error!("fail to send event to sub bus");
        }
    }

    pub async fn send_subscribe(&self, id: WorkerId, tx: Sender<Event>) {
        if self
            .tx
            .send(SubBusData::Subscribe(Subscriber::init(id, tx)))
            .await
            .is_err()
        {
            error!("fail to send subscribe to sub bus");
        }
    }
}

/// 子事件总线
pub struct SubBus {
    type_id: TypeId,
    rx: Receiver<SubBusData>,
    subscribers: HashMap<WorkerId, Subscriber>,
}

impl SubBus {
    pub fn init(type_id: TypeId) -> CopyOfSubBus {
        let (tx, rx) = channel(1024);
        Self {
            type_id,
            rx,
            subscribers: Default::default(),
        }
        .run();
        CopyOfSubBus {
            type_id,
            subscriber_num: 0,
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
                    SubBusData::Event(event) => for subscriber in self.subscribers.values() {},
                }
            }
        });
    }
}
