use crate::value_object::{ChartId, DateTime, Version};

pub type Event = BaseEvent<ChartId, EventData>;

#[derive(Clone, Debug)]
pub struct BaseEvent<I, D> {
    pub at: DateTime,
    pub data: D,
    pub id: I,
    pub version: Version,
}

#[derive(Clone, Debug)]
pub struct Created {
    pub title: String,
}

#[derive(Clone, Debug)]
pub struct Deleted {}

#[derive(Clone, Debug)]
pub struct Updated {
    pub title: String,
}

#[derive(Clone, Debug)]
pub enum EventData {
    Created(Created),
    Deleted(Deleted),
    Updated(Updated),
}
