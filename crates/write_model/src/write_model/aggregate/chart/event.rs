use std::fmt::Display;

use crate::value_object::{ChartId, DateTime, EventId, Version};

pub type Event = BaseEvent<ChartId, EventData>;

#[derive(Clone, Debug)]
pub struct BaseEvent<I: Display, D> {
    pub at: DateTime,
    pub data: D,
    pub id: EventId,
    pub stream_id: I,
    pub version: Version,
}

impl<I: Display, D> BaseEvent<I, D> {
    pub fn new(stream_id: I, data: D, version: Version) -> Self {
        Self {
            at: DateTime::now(),
            data,
            id: EventId::generate(),
            stream_id,
            version,
        }
    }
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
