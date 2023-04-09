#![feature(async_fn_in_trait)]
#![allow(incomplete_features)]
use for_event_bus::worker::{IdentityOfRx, IdentityOfTx, Worker};
use for_event_bus::{Bus, CopyOfBus};
use log::debug;
use std::any::Any;
use std::time::Duration;
use tokio::spawn;
use tokio::time::sleep;

#[tokio::main]
async fn main() {
    custom_utils::logger::logger_stdout_debug();
    let copy_of_bus = Bus::init();
    WorkerA::init(&copy_of_bus).await;
    WorkerB::init(&copy_of_bus).await;
    sleep(Duration::from_secs(5)).await
}

#[derive(Debug)]
struct AEvent;
#[derive(Debug)]
struct BEvent;

struct WorkerA {
    identity: IdentityOfRx,
    identity_tx: IdentityOfTx,
}

impl WorkerA {
    pub async fn init(bus: &CopyOfBus) {
        let (identity, identity_tx) = bus.login().await.unwrap();
        Self {
            identity,
            identity_tx,
        }
        .run();
    }
    fn run(mut self) {
        spawn(async move {
            self.subscribe(AEvent.type_id()).unwrap();
            sleep(Duration::from_secs(1)).await;
            self.dispatch_event(BEvent).unwrap();
            while let Ok(event) = self.identity.recv::<AEvent>().await {
                debug!("WorkerA recv {:?}", event);
                break;
            }
        });
    }
}

struct WorkerB {
    identity: IdentityOfRx,
    identity_tx: IdentityOfTx,
}

impl WorkerB {
    pub async fn init(bus: &CopyOfBus) {
        let (identity, identity_tx) = bus.login().await.unwrap();
        Self {
            identity,
            identity_tx,
        }
        .run();
    }

    fn run(mut self) {
        spawn(async move {
            self.subscribe(BEvent.type_id()).unwrap();
            while let Ok(event) = self.identity.recv::<BEvent>().await {
                debug!("WorkerA recv {:?}", event);
                self.dispatch_event(AEvent).unwrap();
                break;
            }
        });
    }
}

impl Worker for WorkerA {
    fn identity_tx(&self) -> &IdentityOfTx {
        &self.identity_tx
    }
}

impl Worker for WorkerB {
    fn identity_tx(&self) -> &IdentityOfTx {
        &self.identity_tx
    }
}
