use crate::bus::{BusData, Event};

pub trait Merge {
    fn merge(event: Event) -> Result<Self, BusError>
    where
        Self: Sized;

    fn subscribe_types() -> Vec<TypeId>;
}

use crate::bus::BusError;
use crate::worker::identity::{IdentityCommon, IdentityOfRx, IdentityOfTx};

use std::any::{Any, TypeId};
use std::marker::PhantomData;

/// 简单的worker身份标识，只订阅一种事件
pub struct IdentityOfMerge<T: Merge> {
    pub(crate) id: IdentityOfRx,
    pub(crate) phantom: PhantomData<T>,
}

impl<T: Merge> From<IdentityCommon> for IdentityOfMerge<T> {
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

impl<T: Merge + Any + Send + Sync + 'static> IdentityOfMerge<T> {
    pub fn tx(&self) -> IdentityOfTx {
        IdentityOfTx {
            id: self.id.id.clone(),
            tx_data: self.id.tx_data.clone(),
        }
    }
    pub async fn recv(&mut self) -> Result<T, BusError> {
        let event = self.id.recv_event().await?;
        T::merge(event)
    }

    pub fn try_recv(&mut self) -> Result<Option<T>, BusError> {
        let event = self.id.try_recv_event()?;
        match event {
            None => Ok(None),
            Some(event) => Ok(Some(T::merge(event)?)),
        }
    }
    pub(crate) async fn subscribe(&self) -> Result<(), BusError> {
        for type_id in T::subscribe_types() {
            self.id
                .tx_data
                .send(BusData::Subscribe(self.id.id.clone(), type_id))
                .await?;
        }
        Ok(())
    }

    pub async fn dispatch_event<E: Any + Send + Sync + 'static>(
        &self,
        event: E,
    ) -> Result<(), BusError> {
        Ok(self.id.dispatch_event(event).await?)
    }
}
