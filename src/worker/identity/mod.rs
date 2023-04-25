use crate::bus::{BusData, BusError, Event};
use crate::worker::WorkerId;
use log::error;
use std::any::{Any, TypeId};
use std::sync::Arc;
use tokio::sync::mpsc::error::TryRecvError;
use tokio::sync::mpsc::{Receiver, Sender};

mod merge;
mod simple;

pub use simple::IdentityOfSimple;

#[derive(Clone)]
pub struct IdentityOfTx {
    id: WorkerId,
    tx_data: Sender<BusData>,
}

impl IdentityOfTx {
    pub async fn subscribe<T: ?Sized + 'static>(&self) -> Result<(), BusError> {
        Ok(self
            .tx_data
            .send(BusData::Subscribe(self.id.clone(), TypeId::of::<T>()))
            .await?)
    }

    pub async fn dispatch_event<T: Any + Send + Sync + 'static>(
        &self,
        event: T,
    ) -> Result<(), BusError> {
        Ok(self
            .tx_data
            .send(BusData::DispatchEvent(self.id.clone(), Arc::new(event)))
            .await?)
    }
}

/// 通用的worker身份标识，可以订阅多种事件
pub struct IdentityOfRx {
    pub id: WorkerId,
    pub rx_event: Receiver<Event>,
    pub tx_data: Sender<BusData>,
}

pub struct IdentityCommon {
    pub(crate) id: WorkerId,
    pub(crate) rx_event: Receiver<Event>,
    pub(crate) tx_data: Sender<BusData>,
}

impl Drop for IdentityOfRx {
    fn drop(&mut self) {
        if self
            .tx_data
            .try_send(BusData::Drop(self.id.clone()))
            .is_err()
        {
            error!("{:?} send BusData::Drop fail", self.id);
        }
    }
}

impl From<IdentityCommon> for IdentityOfRx {
    fn from(value: IdentityCommon) -> Self {
        let IdentityCommon {
            id,
            rx_event,
            tx_data,
        } = value;
        Self {
            id,
            rx_event,
            tx_data,
        }
    }
}

impl IdentityOfRx {
    // pub fn init(
    //     id: WorkerId,
    //     rx_event: Receiver<Event>,
    //     tx_data: Sender<BusData>,
    // ) -> Self {
    //     Self {
    //         id,
    //         rx_event,
    //         tx_data,
    //     }
    // }

    pub fn tx(&self) -> IdentityOfTx {
        IdentityOfTx {
            id: self.id.clone(),
            tx_data: self.tx_data.clone(),
        }
    }

    // pub async fn recv_event<T: Send + Sync + 'static>(&mut self) -> Option<T> {
    //     self.rx_event.recv().await
    // }
    pub fn rx_event_mut(&mut self) -> &mut Receiver<Event> {
        &mut self.rx_event
    }
    pub async fn recv_event(&mut self) -> Result<Event, BusError> {
        if let Some(event) = self.rx_event.recv().await {
            return Ok(event);
        }
        Err(BusError::ChannelErr)
    }
    pub async fn recv<T: Send + Sync + 'static>(&mut self) -> Result<Arc<T>, BusError> {
        if let Some(event) = self.rx_event.recv().await {
            if let Ok(msg) = event.downcast::<T>() {
                return Ok(msg);
            }
        }
        Err(BusError::ChannelErr)
    }

    pub fn try_recv<T: Send + Sync + 'static>(&mut self) -> Result<Option<Arc<T>>, BusError> {
        match self.rx_event.try_recv() {
            Ok(event) => {
                if let Ok(msg) = event.downcast::<T>() {
                    Ok(Some(msg))
                } else {
                    Ok(None)
                }
            }
            Err(err) => match err {
                TryRecvError::Empty => Ok(None),
                TryRecvError::Disconnected => Err(BusError::ChannelErr),
            },
        }
    }

    // pub fn subscribe(&self, type_id: TypeId) -> Result<(), BusError> {
    //     Ok(self.tx_data.send(BusData::Subscribe(self.id, type_id))?)
    // }

    // pub fn subscribe<T: Any>(&self, worker: &T) -> Result<(), BusError> {
    //     Ok(self
    //         .tx_data
    //         .send(BusData::Subscribe(self.id, worker.type_id()))?)
    // }
    pub async fn subscribe<T: ?Sized + 'static>(&self) -> Result<(), BusError> {
        Ok(self
            .tx_data
            .send(BusData::Subscribe(self.id.clone(), TypeId::of::<T>()))
            .await?)
    }

    pub async fn dispatch_event<T: Any + Send + Sync + 'static>(
        &self,
        event: T,
    ) -> Result<(), BusError> {
        Ok(self
            .tx_data
            .send(BusData::DispatchEvent(self.id.clone(), Arc::new(event)))
            .await?)
    }
}
