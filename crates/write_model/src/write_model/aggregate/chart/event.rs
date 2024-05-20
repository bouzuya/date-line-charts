use crate::value_object::{ChartId, DateTime, Version};

#[derive(Clone, Debug)]
pub struct Created {
    pub at: DateTime,
    pub id: ChartId,
    pub title: String,
    pub version: Version,
}

#[derive(Clone, Debug)]
pub struct Deleted {
    pub at: DateTime,
    pub id: ChartId,
    pub version: Version,
}

#[derive(Clone, Debug)]
pub struct Updated {
    pub at: DateTime,
    pub id: ChartId,
    pub title: String,
    pub version: Version,
}

#[derive(Clone, Debug)]
pub enum Event {
    Created(Created),
    Deleted(Deleted),
    Updated(Updated),
}
