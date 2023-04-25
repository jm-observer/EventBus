// #![feature(associated_type_defaults)]
// #![feature(trait_upcasting)]

mod bus;
mod worker;

pub use bus::{Bus, EntryOfBus};
pub use worker::{identity::IdentityOfSimple, ToWorker};

pub type EventBus = Bus<1000>;
