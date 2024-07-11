use std::str::FromStr as _;

use crate::value_object::{
    ChartId, DataPointId, DateTime, EventId, EventStreamId, Version, YValue,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Event {
    Chart(ChartEvent),
    DataPoint(DataPointEvent),
}

impl Event {
    pub fn at(&self) -> DateTime {
        match self {
            Self::Chart(event) => event.at,
            Self::DataPoint(event) => event.at,
        }
    }

    pub fn id(&self) -> EventId {
        match self {
            Self::Chart(event) => event.id,
            Self::DataPoint(event) => event.id,
        }
    }

    pub fn stream_id(&self) -> EventStreamId {
        match self {
            Self::Chart(event) => EventStreamId::from_str(event.stream_id.to_string().as_str())
                .expect("chart_id to be valid event_stream_id"),
            Self::DataPoint(event) => EventStreamId::from_str(event.stream_id.to_string().as_str())
                .expect("data_point_id to be valid event_stream_id"),
        }
    }

    pub fn version(&self) -> Version {
        match self {
            Self::Chart(event) => event.version,
            Self::DataPoint(event) => event.version,
        }
    }
}

impl From<ChartEvent> for Event {
    fn from(event: ChartEvent) -> Self {
        Self::Chart(event)
    }
}

impl From<DataPointEvent> for Event {
    fn from(event: DataPointEvent) -> Self {
        Self::DataPoint(event)
    }
}

pub type ChartEvent = BaseEvent<ChartEventStream>;

pub type DataPointEvent = BaseEvent<DataPointEventStream>;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BaseEvent<ES: EventStream> {
    pub at: DateTime,
    pub data: ES::Data,
    pub id: EventId,
    pub stream_id: ES::Id,
    pub version: Version,
}

impl<ES: EventStream> BaseEvent<ES> {
    pub fn new(stream_id: ES::Id, data: ES::Data, version: Version) -> Self {
        Self {
            at: DateTime::now(),
            data,
            id: EventId::generate(),
            stream_id,
            version,
        }
    }
}

pub trait EventStream {
    type Data;
    type Id;
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ChartEventStream;

impl EventStream for ChartEventStream {
    type Data = ChartEventData;
    type Id = ChartId;
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DataPointEventStream;

impl EventStream for DataPointEventStream {
    type Data = DataPointEventData;
    type Id = DataPointId;
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ChartEventData {
    Created(ChartCreated),
    Deleted(ChartDeleted),
    Updated(ChartUpdated),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ChartCreated {
    pub title: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ChartDeleted {}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ChartUpdated {
    pub title: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DataPointEventData {
    Created(DataPointCreated),
    Deleted(DataPointDeleted),
    Updated(DataPointUpdated),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DataPointCreated {
    pub value: YValue,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DataPointDeleted {}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DataPointUpdated {
    pub value: YValue,
}

#[cfg(test)]
mod tests {
    use std::str::FromStr as _;

    use super::*;

    #[test]
    fn test_impl_eq_for_event() -> anyhow::Result<()> {
        let at = DateTime::from_str("2021-01-01T00:00:00Z")?;
        let id = EventId::generate();
        let stream_id = ChartId::generate();
        let event1 = Event::Chart(ChartEvent {
            at,
            data: ChartEventData::Created(ChartCreated {
                title: "title1".to_owned(),
            }),
            id,
            stream_id,
            version: Version::new(),
        });
        let event2 = Event::Chart(ChartEvent {
            at,
            data: ChartEventData::Created(ChartCreated {
                title: "title2".to_owned(),
            }),
            id,
            stream_id,
            version: Version::new(),
        });
        assert_eq!(event1, event1);
        assert_ne!(event1, event2);
        Ok(())
    }
}
