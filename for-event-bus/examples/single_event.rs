use for_event_bus::{BusEvent, EntryOfBus, Event, ToWorker};
use for_event_bus::{IdentityOfSimple, SimpleBus};
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
    let copy_of_bus = SimpleBus::init();
    // init worker and subscribe event
    Worker::init(&copy_of_bus).await;
    // init worker and dispatcher event
    WorkerDispatcher::init(&copy_of_bus).await;
    sleep(Duration::from_secs(5)).await
}

#[derive(Debug, Event)]
enum OneEvent<E, C>
where
    E: Event,
    C: Event,
{
    Event(E),
    Command(C),
}
#[derive(Debug, Event)]
struct AEvent;
#[derive(Debug, Event)]
struct Close;

struct Worker {
    identity: IdentityOfSimple<OneEvent<AEvent, Close>>,
}

impl ToWorker for Worker {
    fn name() -> String {
        "Worker".to_string()
    }
}

impl Worker {
    pub async fn init(bus: &EntryOfBus) {
        let identity = bus
            .simple_login::<Self, OneEvent<AEvent, Close>>()
            .await
            .unwrap();
        Self { identity }.run();
    }
    fn run(mut self) {
        spawn(async move {
            while let Ok(event) = self.identity.recv().await {
                match event.as_ref() {
                    BusEvent::Event(event) => {
                        debug!("WorkerA recv {:?}", event);
                    }
                    BusEvent::Command(_) => {
                        debug!("recv close");
                        break;
                    }
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
        let identity = bus.simple_login::<Self, ()>().await.unwrap();
        Self { identity }.run();
    }
    fn run(self) {
        spawn(async move {
            self.identity
                .dispatch_event(BusEvent::<AEvent, Close>::Event(AEvent))
                .await
                .unwrap();
            self.identity
                .dispatch_event(BusEvent::<AEvent, Close>::Command(Close))
                .await
                .unwrap();
        });
    }
}
