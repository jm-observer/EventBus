use crate::{BusData, BusError, Event};
use log::error;
use std::any::{Any, TypeId};
use std::collections::HashSet;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use async_trait::async_trait;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};

pub struct IdentityOfTx {
    id: WorkerId,
    tx_data: UnboundedSender<BusData>,
}

impl IdentityOfTx {
    pub fn init(id: WorkerId, tx_data: UnboundedSender<BusData>) -> Self {
        Self { id, tx_data }
    }

    pub fn subscribe(&self, type_id: TypeId) -> Result<(), BusError> {
        Ok(self.tx_data.send(BusData::Subscribe(self.id, type_id))?)
    }

    pub fn dispatch_event<T: Any + Send + Sync + 'static>(&self, event: T) -> Result<(), BusError> {
        Ok(self
            .tx_data
            .send(BusData::DispatchEvent(self.id, Arc::new(event)))?)
    }
}

pub struct IdentityOfRx {
    id: WorkerId,
    rx_event: UnboundedReceiver<Event>,
    tx_data: UnboundedSender<BusData>,
}

impl Drop for IdentityOfRx {
    fn drop(&mut self) {
        if self.tx_data.send(BusData::Drop(self.id)).is_err() {
            error!("{:?} send BusData::Drop fail", self.id);
        }
    }
}

impl IdentityOfRx {
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

    // pub async fn recv_event<T: Send + Sync + 'static>(&mut self) -> Option<T> {
    //     self.rx_event.recv().await
    // }

    pub async fn recv<T: Send + Sync + 'static>(&mut self) -> Result<Arc<T>, BusError> {
        while let Some(event) = self.rx_event.recv().await {
            if let Ok(msg) = event.downcast::<T>() {
                return Ok(msg);
            }
        }
        Err(BusError::ChannelErr)
    }

    pub fn subscribe(&self, type_id: TypeId) -> Result<(), BusError> {
        Ok(self.tx_data.send(BusData::Subscribe(self.id, type_id))?)
    }

    pub fn dispatch_event<T: Any + Send + Sync + 'static>(&self, event: T) -> Result<(), BusError> {
        Ok(self
            .tx_data
            .send(BusData::DispatchEvent(self.id, Arc::new(event)))?)
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
    // type EventType: Send + Sync + 'static = ();

    fn identity_tx(&self) -> &IdentityOfTx;

    fn subscribe(&self, type_id: TypeId) -> Result<(), BusError> {
        self.identity_tx().subscribe(type_id)
    }

    fn dispatch_event<T: Any + Send + Sync + 'static>(&mut self, event: T) -> Result<(), BusError> {
        let identity = self.identity_tx();
        identity.dispatch_event(event)
    }

    /*
    async fn recv(&mut self) -> Option<Arc<Self::EventType>> {
        while let Some(event) = self.identity_rx_mut().recv_event().await {
            if let Ok(msg) = event.downcast::<Self::EventType>() {
                return Some(msg);
            }
        }
        None
    }
    */
}
