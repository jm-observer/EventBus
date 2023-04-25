use for_event_bus::bus::{Bus, EntryOfBus};
use for_event_bus::identity::simple::IdentityOfSimple;
use for_event_bus::identity::{IdentityOfRx, IdentityOfTx};
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
enum Event<E, C>
where
    E: Any + Send + Sync + 'static,
    C: Any + Send + Sync + 'static,
{
    Event(E),
    Command(C),
}
#[derive(Debug, Clone)]
struct AEvent;
#[derive(Debug, Clone)]
struct Close;

#[derive(Debug, Clone)]
enum MergeEvent {
    AEvent(AEvent),
    Close(Close),
}

trait Merge {
    fn merge(event: for_event_bus::bus::Event) -> Result<Self, ()>
    where
        Self: Sized;
}

impl Merge for MergeEvent {
    fn merge(event: for_event_bus::bus::Event) -> Result<Self, ()> {
        if let Ok(a_event) = event.clone().downcast::<AEvent>() {
            Ok(Self::AEvent(a_event.as_ref().clone()))
        } else if let Ok(a_event) = event.clone().downcast::<Close>() {
            Ok(Self::Close(a_event.as_ref().clone()))
        } else {
            Err(())
        }
    }
}

struct Worker {
    identity: IdentityOfRx,
}

impl Worker {
    pub async fn init(bus: &EntryOfBus) {
        let identity = bus.login().await.unwrap();
        identity.subscribe::<AEvent>().unwrap();
        identity.subscribe::<Close>().unwrap();
        Self { identity }.run();
    }
    fn run(mut self) {
        spawn(async move {
            while let Ok(event) = self.identity.recv_event().await {
                if let Ok(event) = MergeEvent::merge(event) {
                    debug!("{:?}", event);
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
