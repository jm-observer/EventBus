use for_event_bus::bus::{Bus, EntryOfBus};
use for_event_bus::identity::simple::IdentityOfSimple;
use for_event_bus::identity::IdentityOfRx;
use log::debug;
use std::any::Any;
use std::time::Duration;
use tokio::spawn;
use tokio::time::sleep;

#[tokio::main]
async fn main() {
    // log
    custom_utils::logger::logger_stdout_debug();
    // init event bus
    let copy_of_bus = Bus::init();
    // init worker and subscribe event
    Worker::init(&copy_of_bus).await;
    // init worker and dispatcher event
    WorkerDispatcher::init(&copy_of_bus).await;
    sleep(Duration::from_secs(5)).await
}

#[derive(Debug)]
struct AEvent;
#[derive(Debug)]
struct Close;

struct Worker {
    identity: IdentityOfRx,
}

impl Worker {
    pub async fn init(bus: &EntryOfBus) {
        let identity = bus.login().await.unwrap();
        Self { identity }.run();
    }
    fn run(mut self) {
        spawn(async move {
            self.identity.subscribe::<AEvent>().unwrap();
            self.identity.subscribe::<Close>().unwrap();
            while let Ok(event) = self.identity.recv_event().await {
                if let Ok(msg) = event.clone().downcast::<AEvent>() {
                    debug!("recv {:?}", msg);
                } else if let Ok(msg) = event.clone().downcast::<Close>() {
                    debug!("recv close");
                    break;
                }
            }
        });
    }
}

struct WorkerDispatcher {
    identity: IdentityOfSimple<()>,
}

impl WorkerDispatcher {
    pub async fn init(bus: &EntryOfBus) {
        let identity = bus.simple_login().await.unwrap();
        Self { identity }.run();
    }
    fn run(self) {
        spawn(async move {
            self.identity.dispatch_event(AEvent).unwrap();
            self.identity.dispatch_event(Close).unwrap();
        });
    }
}
