use log::error;
use std::any::TypeId;
use std::collections::HashSet;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use crate::bus::Event;
use tokio::sync::mpsc::Sender;

pub mod identity;

static ID: AtomicUsize = AtomicUsize::new(0);

#[derive(Debug, Eq, PartialEq, Clone, Hash)]
pub struct WorkerId {
    id: usize,
    name: Arc<String>,
}

impl WorkerId {
    pub fn init(name: String) -> Self {
        let id = ID.fetch_add(1, Ordering::Release);
        Self {
            id,
            name: Arc::new(name),
        }
    }
}

pub(crate) struct Worker {
    id: WorkerId,
    tx: Sender<Event>,
}

impl Worker {
    pub fn id(&self) -> WorkerId {
        self.id.clone()
    }
    pub async fn send(&self, event: Event) {
        if self.tx.send(event).await.is_err() {
            error!("send event to {:?} fail", self.id);
        }
    }
}
pub(crate) struct CopyOfWorker {
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
        self.id.clone()
    }

    pub fn init_subscriber(&self) -> Worker {
        Worker {
            id: self.id.clone(),
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

// #[async_trait]
// pub trait Worker {
//     // type EventType: Send + Sync + 'static = ();
//
//     fn identity_tx(&self) -> &IdentityOfTx;
//
//     fn subscribe<T: ?Sized + 'static>(&self) -> Result<(), BusError> {
//         self.identity_tx().subscribe::<T>()
//     }
//
//     fn dispatch_event<T: Any + Send + Sync + 'static>(&mut self, event: T) -> Result<(), BusError> {
//         let identity = self.identity_tx();
//         identity.dispatch_event(event)
//     }
//
//     /*
//     async fn recv(&mut self) -> Option<Arc<Self::EventType>> {
//         while let Some(event) = self.identity_rx_mut().recv_event().await {
//             if let Ok(msg) = event.downcast::<Self::EventType>() {
//                 return Some(msg);
//             }
//         }
//         None
//     }
//     */
// }

pub trait ToWorker {
    fn name() -> String;
}
