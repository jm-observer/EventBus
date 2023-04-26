// #![feature(associated_type_defaults)]
// #![feature(trait_upcasting)]

mod bus;
mod worker;

pub use bus::{Bus, BusError, EntryOfBus, Event};
pub use worker::{
    identity::{IdentityOfMerge, IdentityOfSimple, Merge},
    ToWorker,
};

pub type SimpleBus = Bus<1000>;
