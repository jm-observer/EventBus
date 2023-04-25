use crate::bus::{BusData, BusError, Event};
use crate::worker::identity::{IdentityCommon, IdentityOfRx, IdentityOfTx};
use crate::worker::WorkerId;
use log::error;
use std::any::{Any, TypeId};
use std::marker::PhantomData;
use std::sync::Arc;
use tokio::sync::mpsc::error::TryRecvError;
use tokio::sync::mpsc::{Receiver, Sender};

/// 简单的worker身份标识，只订阅一种事件
pub struct IdentityOfSimple<T> {
    pub(crate) id: IdentityOfRx,
    pub(crate) phantom: PhantomData<T>,
}

impl<T> From<IdentityCommon> for IdentityOfSimple<T> {
    fn from(value: IdentityCommon) -> Self {
        let IdentityCommon {
            id,
            rx_event,
            tx_data,
        } = value;
        let id = IdentityOfRx {
            id,
            rx_event,
            tx_data,
        };
        Self {
            id,
            phantom: Default::default(),
        }
    }
}

impl<T: Any + Send + Sync + 'static> IdentityOfSimple<T> {
    pub fn tx(&self) -> IdentityOfTx {
        IdentityOfTx {
            id: self.id.id.clone(),
            tx_data: self.id.tx_data.clone(),
        }
    }
    pub async fn recv(&mut self) -> Result<Arc<T>, BusError> {
        self.id.recv::<T>().await
    }

    pub fn try_recv(&mut self) -> Result<Option<Arc<T>>, BusError> {
        self.id.try_recv::<T>()
    }
    pub(crate) async fn subscribe(&self) -> Result<(), BusError> {
        Ok(self.id.subscribe::<T>().await?)
    }

    pub async fn dispatch_event<E: Any + Send + Sync + 'static>(
        &self,
        event: E,
    ) -> Result<(), BusError> {
        Ok(self.id.dispatch_event(event).await?)
    }
}
