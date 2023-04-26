use for_event_bus::{EntryOfBus, IdentityOfRx, IdentityOfSimple, SimpleBus, ToWorker};
use log::debug;
use std::time::Duration;
use tokio::spawn;
use tokio::time::sleep;

#[tokio::main]
async fn main() {
    // log
    custom_utils::logger::logger_stdout_debug();
    // init event bus
    let copy_of_bus = SimpleBus::init();
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

impl ToWorker for Worker {
    fn name() -> String {
        "Worker".to_string()
    }
}

impl Worker {
    pub async fn init(bus: &EntryOfBus) {
        let identity = bus.login::<Worker>().await.unwrap();
        Self { identity }.run();
    }
    fn run(mut self) {
        spawn(async move {
            self.identity.subscribe::<AEvent>().await.unwrap();
            self.identity.subscribe::<Close>().await.unwrap();
            while let Ok(event) = self.identity.recv_event().await {
                if let Ok(msg) = event.clone().downcast::<AEvent>() {
                    debug!("recv {:?}", msg);
                } else if let Ok(_) = event.clone().downcast::<Close>() {
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

impl ToWorker for WorkerDispatcher {
    fn name() -> String {
        "WorkerDispatcher".to_string()
    }
}

impl WorkerDispatcher {
    pub async fn init(bus: &EntryOfBus) {
        let identity = bus.simple_login::<WorkerDispatcher, ()>().await.unwrap();
        Self { identity }.run();
    }
    fn run(self) {
        spawn(async move {
            self.identity.dispatch_event(AEvent).await.unwrap();
            self.identity.dispatch_event(Close).await.unwrap();
        });
    }
}
