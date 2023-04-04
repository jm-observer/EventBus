use std::any::TypeId;
use std::sync::Arc;
use tokio::sync::mpsc::{Receiver, Sender};
use crate::{Event, WorkerId};

struct Subscriber {
    id: WorkerId,
    tx: Sender<Arc<Event>>,
}

pub enum SubBusData {
    Subscribe(Subscriber),
    Unsubscribe(WorkerId),
}

pub struct CopyOfSubBus {
    type_id: TypeId,
    subscriber_num: usize,
    tx: Sender<SubBusData>
}

/// 子事件总线
struct SubBus {
    rx: Receiver<SubBusData>,
    subscribers: Vec<Subscriber>
}