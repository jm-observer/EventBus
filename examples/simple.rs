use event_bus::worker::IdentityOfWorker;
use event_bus::{Bus, CopyOfBus};
use log::debug;
use std::any::Any;
use std::sync::Arc;
use std::time::Duration;
use tokio::spawn;
use tokio::time::sleep;

#[tokio::main]
async fn main() {
    custom_utils::logger::logger_stdout_debug();
    let copy_of_bus = Bus::init();
    WorkerA::init(&copy_of_bus).await;
    WorkerB::init(&copy_of_bus).await;
    sleep(Duration::from_secs(1)).await;
    copy_of_bus.send_event(Arc::new(AEvent)).await.unwrap();
    copy_of_bus.send_event(Arc::new(BEvent)).await.unwrap();
    sleep(Duration::from_secs(5)).await
}

#[derive(Debug)]
struct AEvent;
#[derive(Debug)]
struct BEvent;

struct WorkerA {
    identity: IdentityOfWorker,
}

impl WorkerA {
    pub async fn init(bus: &CopyOfBus) {
        let identity = bus.login().await.unwrap();
        Self { identity }.run();
    }
    fn run(mut self) {
        spawn(async move {
            self.identity.subscribe(AEvent.type_id()).await.unwrap();
            while let Some(event) = self.identity.recv_event().await {
                debug!("WorkerA recv {:?}", event.as_ref().type_id());
                if let Some(a) = event.as_ref().downcast_ref::<AEvent>() {
                    debug!("WorkerA recv {:?}", a);
                }
            }
        });
    }
}

struct WorkerB {
    identity: IdentityOfWorker,
}

impl WorkerB {
    pub async fn init(bus: &CopyOfBus) {
        let identity = bus.login().await.unwrap();
        Self { identity }.run();
    }
    fn run(mut self) {
        spawn(async move {
            self.identity.subscribe(BEvent.type_id()).await.unwrap();
            while let Some(event) = self.identity.recv_event().await {
                debug!("WorkerB recv {:?}", event.as_ref().type_id());
                if let Some(a) = event.as_ref().downcast_ref::<BEvent>() {
                    debug!("WorkerB recv {:?}", a);
                }
            }
        });
    }
}
