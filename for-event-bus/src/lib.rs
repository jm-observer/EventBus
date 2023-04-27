// #![feature(associated_type_defaults)]
// #![feature(trait_upcasting)]

mod bus;
mod worker;

pub use bus::{Bus, BusError, EntryOfBus, Event};
pub use worker::{
    identity::{IdentityOfMerge, IdentityOfRx, IdentityOfSimple, Merge},
    ToWorker,
};

pub use for_event_bus_derive::Merge;

pub type SimpleBus = Bus<1000>;
