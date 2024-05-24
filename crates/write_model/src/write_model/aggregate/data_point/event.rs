use crate::{
    aggregate::chart::event::BaseEvent,
    value_object::{DataPointId, YValue},
};

pub type Event = BaseEvent<DataPointId, EventData>;

#[derive(Clone, Debug)]
pub struct Created {
    pub value: YValue,
}

#[derive(Clone, Debug)]
pub struct Deleted {}

#[derive(Clone, Debug)]
pub struct Updated {
    pub value: YValue,
}

#[derive(Clone, Debug)]
pub enum EventData {
    Created(Created),
    Deleted(Deleted),
    Updated(Updated),
}
