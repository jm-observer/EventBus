use for_event_bus::worker::{IdentityOfRx, IdentityOfSimple};
use for_event_bus::{Bus, CopyOfBus};
use log::debug;
use std::time::Duration;
use tokio::spawn;
use tokio::time::sleep;

#[tokio::main]
async fn main() {
    custom_utils::logger::logger_stdout_debug();
    let copy_of_bus = Bus::init();
    WorkerB::init(&copy_of_bus).await;
    sleep(Duration::from_secs(1)).await;
    WorkerA::init(&copy_of_bus).await;
    sleep(Duration::from_secs(5)).await
}

#[derive(Debug)]
struct AEvent;
#[derive(Debug)]
struct BEvent;

struct WorkerA {
    identity: IdentityOfSimple<AEvent>,
}

impl WorkerA {
    pub async fn init(bus: &CopyOfBus) {
        let identity = bus.simple_login().await.unwrap();
        Self { identity }.run();
    }
    fn run(mut self) {
        spawn(async move {
            self.identity.dispatch_event(BEvent).unwrap();
            while let Ok(event) = self.identity.recv().await {
                debug!("WorkerA recv {:?}", event);
                break;
            }
        });
    }
}

struct WorkerB {
    identity: IdentityOfRx,
}

impl WorkerB {
    pub async fn init(bus: &CopyOfBus) {
        let identity = bus.login().await.unwrap();
        Self { identity }.run();
    }

    fn run(mut self) {
        spawn(async move {
            self.identity.subscribe::<BEvent>().unwrap();
            self.identity.subscribe::<AEvent>().unwrap();
            while let Ok(event) = self.identity.recv_event().await {
                if let Ok(a) = event.clone().downcast::<AEvent>() {
                    debug!("WorkerB recv {:?}", a);
                    break;
                } else if let Ok(b) = event.clone().downcast::<BEvent>() {
                    debug!("WorkerB recv {:?}", b);
                    self.identity.dispatch_event(AEvent).unwrap();
                }
            }
        });
    }
}
