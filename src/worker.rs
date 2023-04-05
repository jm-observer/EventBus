use crate::{BusData, Event};
use anyhow::{anyhow, Result};
use log::error;
use std::any::TypeId;
use std::collections::HashSet;
use std::sync::atomic::{AtomicUsize, Ordering};

use tokio::sync::mpsc::{Receiver, Sender};

pub struct IdentityOfWorker {
    id: WorkerId,
    rx_event: Receiver<Event>,
    tx_data: Sender<BusData>,
}

impl IdentityOfWorker {
    pub fn init(id: WorkerId, rx_event: Receiver<Event>, tx_data: Sender<BusData>) -> Self {
        Self {
            id,
            rx_event,
            tx_data,
        }
    }

    pub async fn recv_event(&mut self) -> Option<Event> {
        self.rx_event.recv().await
    }

    pub async fn subscribe(&mut self, type_id: TypeId) -> Result<()> {
        self.tx_data
            .send(BusData::Subscribe(self.id, type_id))
            .await
            .map_err(|_| anyhow!("fail to contact bus"))
    }
}

impl Drop for IdentityOfWorker {
    fn drop(&mut self) {
        // todo!()
    }
}

static ID: AtomicUsize = AtomicUsize::new(0);

#[derive(Debug, Eq, PartialEq, Copy, Clone, Hash)]
pub struct WorkerId(usize);

impl Default for WorkerId {
    fn default() -> Self {
        Self(ID.fetch_add(1, Ordering::Release))
    }
}

pub struct Subscriber {
    id: WorkerId,
    tx: Sender<Event>,
}

impl Subscriber {
    pub fn init(id: WorkerId, tx: Sender<Event>) -> Self {
        Self { id, tx }
    }
    pub fn id(&self) -> WorkerId {
        self.id
    }
    pub async fn send(&self, event: Event) {
        if self.tx.send(event).await.is_err() {
            error!("send event to {:?} fail", self.id);
        }
    }
}
pub struct CopyOfWorker {
    id: WorkerId,
    tx_event: Sender<Event>,
    subscribe_events: HashSet<TypeId>,
}
impl CopyOfWorker {
    pub fn init(id: WorkerId, tx_event: Sender<Event>) -> Self {
        Self {
            id,
            tx_event,
            subscribe_events: Default::default(),
        }
    }
    pub fn id(&self) -> WorkerId {
        self.id
    }
    pub fn tx(&self) -> Sender<Event> {
        self.tx_event.clone()
    }
    pub fn init_subscriber(&self) -> Subscriber {
        Subscriber {
            id: self.id,
            tx: self.tx_event.clone(),
        }
    }
    pub fn subscribe_event(&mut self, ty_id: TypeId) {
        self.subscribe_events.insert(ty_id);
    }
    pub fn subscribe_events(&self) -> std::collections::hash_set::Iter<'_, TypeId> {
        self.subscribe_events.iter()
    }
}
