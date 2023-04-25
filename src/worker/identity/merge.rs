use crate::bus::Event;

pub trait Merge {
    fn merge(event: Event) -> Result<Self, ()>
    where
        Self: Sized;
}
