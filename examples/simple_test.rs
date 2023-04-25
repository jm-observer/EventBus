use for_event_bus::bus::{Bus, EntryOfBus};
use for_event_bus::identity::simple::IdentityOfSimple;
use log::LevelFilter::Info;
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use tokio::spawn;
use tokio::sync::Barrier;
use tokio::time::sleep;

/// cargo run --package for_event_bus --example simple_test --release
/// 测试单事件分发，单worker接收的耗时
#[tokio::main]
async fn main() {
    // log
    custom_utils::logger::logger_stdout(Info);
    let barrier = Arc::new(Barrier::new(2));
    // init event bus
    let copy_of_bus = Bus::init();
    // init worker and subscribe event
    Worker::init(&copy_of_bus, barrier.clone()).await;
    // init worker and dispatcher event
    WorkerDispatcher::init(&copy_of_bus, barrier).await;
    sleep(Duration::from_secs(5)).await
}

struct AEvent;

const EVENT_NUM: usize = 10_000_000;

struct Worker {
    identity: IdentityOfSimple<AEvent>,
    barrier: Arc<Barrier>,
}

impl Worker {
    pub async fn init(bus: &EntryOfBus, barrier: Arc<Barrier>) {
        let identity = bus.simple_login().await.unwrap();
        Self { identity, barrier }.run();
    }
    fn run(mut self) {
        spawn(async move {
            let mut count = 0;
            self.barrier.wait().await;
            let start = SystemTime::now();
            while let Ok(_) = self.identity.recv().await {
                count += 1;
                if count == EVENT_NUM {
                    break;
                }
            }
            let end = start.elapsed().unwrap();
            println!("{:?}", end);
        });
    }
}

struct WorkerDispatcher {
    identity: IdentityOfSimple<()>,
    barrier: Arc<Barrier>,
}

impl WorkerDispatcher {
    pub async fn init(bus: &EntryOfBus, barrier: Arc<Barrier>) {
        let identity = bus.simple_login().await.unwrap();
        Self { identity, barrier }.run();
    }
    fn run(self) {
        spawn(async move {
            self.barrier.wait().await;
            for _ in 0..EVENT_NUM {
                self.identity.dispatch_event(AEvent).unwrap();
            }
        });
    }
}
