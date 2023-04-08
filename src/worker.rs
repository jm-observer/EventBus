use crate::{BusData, CopyOfBus, Event};
use anyhow::{anyhow, Result};
use log::error;
use std::any::{Any, TypeId};
use std::collections::HashSet;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use async_trait::async_trait;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};

pub struct IdentityOfWorker {
    id: WorkerId,
    rx_event: UnboundedReceiver<Event>,
    tx_data: UnboundedSender<BusData>,
}

impl Drop for IdentityOfWorker {
    fn drop(&mut self) {
        if self.tx_data.send(BusData::Drop(self.id)).is_err() {
            error!("{:?} send BusData::Drop fail", self.id);
        }
    }
}

impl IdentityOfWorker {
    pub fn init(
        id: WorkerId,
        rx_event: UnboundedReceiver<Event>,
        tx_data: UnboundedSender<BusData>,
    ) -> Self {
        Self {
            id,
            rx_event,
            tx_data,
        }
    }

    pub async fn recv_event(&mut self) -> Option<Event> {
        self.rx_event.recv().await
    }

    pub fn subscribe(&self, type_id: TypeId) -> Result<()> {
        self.tx_data
            .send(BusData::Subscribe(self.id, type_id))
            .map_err(|_| anyhow!("fail to contact bus"))
    }

    pub fn dispatch_event<T: Any + Send + Sync + 'static>(&self, event: T) -> Result<()> {
        self.tx_data
            .send(BusData::DispatchEvent(self.id, Arc::new(event)))
            .map_err(|_| anyhow!("fail to contact bus"))
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
    tx: UnboundedSender<Event>,
}

impl Subscriber {
    pub fn init(id: WorkerId, tx: UnboundedSender<Event>) -> Self {
        Self { id, tx }
    }
    pub fn id(&self) -> WorkerId {
        self.id
    }
    pub async fn send(&self, event: Event) {
        if self.tx.send(event).is_err() {
            error!("send event to {:?} fail", self.id);
        }
    }
}
pub struct CopyOfWorker {
    id: WorkerId,
    tx_event: UnboundedSender<Event>,
    subscribe_events: HashSet<TypeId>,
}
impl CopyOfWorker {
    pub fn init(id: WorkerId, tx_event: UnboundedSender<Event>) -> Self {
        Self {
            id,
            tx_event,
            subscribe_events: Default::default(),
        }
    }
    pub fn id(&self) -> WorkerId {
        self.id
    }
    pub fn tx(&self) -> UnboundedSender<Event> {
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

#[async_trait]
pub trait Worker {
    async fn login(bus: &CopyOfBus) -> Result<IdentityOfWorker> {
        bus.login().await
    }

    fn identity(&self) -> &IdentityOfWorker;

    fn subscribe(&self, type_id: TypeId) -> Result<()> {
        self.identity().subscribe(type_id)
    }

    fn dispatch_event<T: Any + Send + Sync + 'static>(&mut self, event: T) -> Result<()> {
        let identity = self.identity();
        identity.dispatch_event(event)
    }
}
