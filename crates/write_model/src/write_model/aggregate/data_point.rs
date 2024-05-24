pub mod event;

use crate::value_object::{ChartId, DataPointId, DateTime, Version, XValue, YValue};

pub use self::event::Event;
use self::event::{Created, Deleted, EventData, Updated};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("multiple created event")]
    MultipleCreatedEvent,
    #[error("no created event")]
    NoCreatedEvent,
    #[error("version overflow")]
    VersionOverflow,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DataPoint {
    created_at: DateTime,
    deleted_at: Option<DateTime>,
    id: DataPointId,
    version: Version,
    y_value: YValue,
}

impl DataPoint {
    pub fn create(
        chart_id: ChartId,
        x_value: XValue,
        y_value: YValue,
    ) -> Result<(Self, Vec<Event>), Error> {
        let events = vec![Event::new(
            DataPointId::new(chart_id, x_value),
            EventData::Created(Created { value: y_value }),
            Version::new(),
        )];
        let state = Self {
            created_at: events[0].at,
            deleted_at: None,
            id: events[0].stream_id,
            version: events[0].version,
            y_value,
        };
        Ok((state, events))
    }

    pub fn from_events(events: &[Event]) -> Result<Self, Error> {
        let mut state = match events.first() {
            None => return Err(Error::NoCreatedEvent),
            Some(Event {
                at,
                data: EventData::Created(event),
                id: _,
                stream_id,
                version,
            }) => Self {
                created_at: *at,
                deleted_at: None,
                id: *stream_id,
                version: *version,
                y_value: event.value,
            },
            Some(_) => return Err(Error::NoCreatedEvent),
        };
        state.apply_events(&events[1..])?;
        Ok(state)
    }

    pub fn reconstruct(
        created_at: DateTime,
        deleted_at: Option<DateTime>,
        id: DataPointId,
        version: Version,
        y_value: YValue,
    ) -> Self {
        Self {
            created_at,
            deleted_at,
            id,
            version,
            y_value,
        }
    }

    pub fn chart_id(&self) -> ChartId {
        self.id.chart_id()
    }

    pub fn created_at(&self) -> DateTime {
        self.created_at
    }

    pub fn delete(&self) -> Result<(Self, Vec<Event>), Error> {
        let events = vec![Event::new(
            self.id,
            EventData::Deleted(Deleted {}),
            self.version.next().map_err(|_| Error::VersionOverflow)?,
        )];
        let mut state = self.clone();
        state.apply_events(&events)?;
        Ok((state, events))
    }

    pub fn deleted_at(&self) -> Option<DateTime> {
        self.deleted_at
    }

    pub fn id(&self) -> DataPointId {
        self.id
    }

    pub fn x_value(&self) -> XValue {
        self.id.x_value()
    }

    pub fn y_value(&self) -> YValue {
        self.y_value
    }

    pub fn update(&self, y_value: YValue) -> Result<(Self, Vec<Event>), Error> {
        let events = vec![Event::new(
            self.id,
            EventData::Updated(Updated { value: y_value }),
            self.version.next().map_err(|_| Error::VersionOverflow)?,
        )];
        let mut state = self.clone();
        state.apply_events(&events)?;
        Ok((state, events))
    }

    pub fn version(&self) -> Version {
        self.version
    }

    fn apply_events(&mut self, events: &[Event]) -> Result<(), Error> {
        for event in events {
            let at = event.at;
            let version = event.version;
            match &event.data {
                EventData::Created(_) => return Err(Error::MultipleCreatedEvent),
                EventData::Updated(e) => {
                    self.version = version;
                    self.y_value = e.value;
                }
                EventData::Deleted(_) => {
                    self.deleted_at = Some(at);
                    self.version = version;
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::*;

    #[test]
    fn test() -> anyhow::Result<()> {
        let mut all_events = vec![];
        let chart_id = ChartId::generate();
        let (state, events) = DataPoint::create(
            chart_id,
            XValue::from_str("2020-01-02")?,
            YValue::from(123_u32),
        )?;
        all_events.extend(events);
        assert_eq!(DataPoint::from_events(&all_events)?, state);
        assert!(state.deleted_at.is_none());
        assert_eq!(state.chart_id(), chart_id);
        assert_eq!(state.x_value(), XValue::from_str("2020-01-02")?);
        assert_eq!(state.y_value(), YValue::from(123_u32));
        let (state, events) = state.update(YValue::from(456_u32))?;
        all_events.extend(events);
        assert_eq!(state.y_value(), YValue::from(456_u32));
        assert_eq!(DataPoint::from_events(&all_events)?, state);
        let (state, events) = state.delete()?;
        all_events.extend(events);
        assert_eq!(DataPoint::from_events(&all_events)?, state);
        assert!(state.deleted_at.is_some());
        Ok(())
    }
}
