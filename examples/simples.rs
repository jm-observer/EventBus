use for_event_bus::{BusError, IdentityOfSimple, Merge, ToWorker};
use for_event_bus::{EntryOfBus, IdentityOfMerge, SimpleBus};
use log::debug;
use std::any::TypeId;
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
    {
        Worker::init(&copy_of_bus).await;
        // init worker and dispatcher event
        WorkerDispatcher::init(&copy_of_bus).await;
        sleep(Duration::from_secs(5)).await
    }
    sleep(Duration::from_secs(5)).await
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

impl Merge for MergeEvent {
    fn merge(event: for_event_bus::Event) -> Result<Self, BusError>
    where
        Self: Sized,
    {
        if let Ok(a_event) = event.clone().downcast::<AEvent>() {
            Ok(Self::AEvent(a_event.as_ref().clone()))
        } else if let Ok(a_event) = event.clone().downcast::<Close>() {
            Ok(Self::Close(a_event.as_ref().clone()))
        } else {
            Err(BusError::ChannelErr)
        }
    }

    fn subscribe_types() -> Vec<TypeId> {
        vec![TypeId::of::<AEvent>(), TypeId::of::<Close>()]
    }
}

struct Worker {
    identity: IdentityOfMerge<MergeEvent>,
}

impl ToWorker for Worker {
    fn name() -> String {
        "Worker".to_string()
    }
}

impl Worker {
    pub async fn init(bus: &EntryOfBus) {
        let identity = bus.merge_login::<Worker, MergeEvent>().await.unwrap();
        Self { identity }.run();
    }
    fn run(mut self) {
        spawn(async move {
            while let Ok(event) = self.identity.recv().await {
                debug!("{:?}", event);
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
