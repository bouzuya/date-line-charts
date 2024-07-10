use crate::value_object::{ChartId, DataPointId, DateTime, Version, XValue, YValue};

use crate::event::{
    DataPointCreated, DataPointDeleted, DataPointEvent, DataPointEventData, DataPointUpdated,
};

#[derive(Debug, PartialEq, thiserror::Error)]
pub enum Error {
    #[error("already deleted")]
    AlreadyDeleted,
    #[error("multiple created event")]
    MultipleCreatedEvent,
    #[error("no created event")]
    NoCreatedEvent,
    #[error("version overflow")]
    VersionOverflow,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DataPoint {
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
    ) -> Result<(Self, Vec<DataPointEvent>), Error> {
        let events = vec![DataPointEvent::new(
            DataPointId::new(chart_id, x_value),
            DataPointEventData::Created(DataPointCreated { value: y_value }),
            Version::new(),
        )];
        let state = Self {
            deleted_at: None,
            id: events[0].stream_id,
            version: events[0].version,
            y_value,
        };
        Ok((state, events))
    }

    pub fn from_events(events: &[DataPointEvent]) -> Result<Self, Error> {
        let mut state = match events.first() {
            None => return Err(Error::NoCreatedEvent),
            Some(DataPointEvent {
                at: _,
                data: DataPointEventData::Created(event),
                id: _,
                stream_id,
                version,
            }) => Self {
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
        deleted_at: Option<DateTime>,
        id: DataPointId,
        version: Version,
        y_value: YValue,
    ) -> Self {
        Self {
            deleted_at,
            id,
            version,
            y_value,
        }
    }

    pub fn chart_id(&self) -> ChartId {
        self.id.chart_id()
    }

    pub fn delete(&self) -> Result<(Self, Vec<DataPointEvent>), Error> {
        if self.deleted_at.is_some() {
            return Err(Error::AlreadyDeleted);
        }
        let events = vec![DataPointEvent::new(
            self.id,
            DataPointEventData::Deleted(DataPointDeleted {}),
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

    pub fn update(&self, y_value: YValue) -> Result<(Self, Vec<DataPointEvent>), Error> {
        if self.deleted_at.is_some() {
            return Err(Error::AlreadyDeleted);
        }
        let events = vec![DataPointEvent::new(
            self.id,
            DataPointEventData::Updated(DataPointUpdated { value: y_value }),
            self.version.next().map_err(|_| Error::VersionOverflow)?,
        )];
        let mut state = self.clone();
        state.apply_events(&events)?;
        Ok((state, events))
    }

    pub fn version(&self) -> Version {
        self.version
    }

    fn apply_events(&mut self, events: &[DataPointEvent]) -> Result<(), Error> {
        for event in events {
            let at = event.at;
            let version = event.version;
            match &event.data {
                DataPointEventData::Created(_) => return Err(Error::MultipleCreatedEvent),
                DataPointEventData::Updated(e) => {
                    self.version = version;
                    self.y_value = e.value;
                }
                DataPointEventData::Deleted(_) => {
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

    #[test]
    fn test_delete() -> anyhow::Result<()> {
        let (before_state, before_events) = build_data_point()?;
        let (deleted, events) = before_state.delete()?;
        assert_eq!(deleted.chart_id(), before_state.chart_id());
        assert!(deleted.deleted_at().is_some());
        assert_eq!(deleted.id(), before_state.id());
        assert_eq!(deleted.x_value(), before_state.x_value());
        assert_eq!(deleted.y_value(), before_state.y_value());
        let all_events = {
            let mut e = before_events.clone();
            e.extend(events);
            e
        };
        assert_eq!(DataPoint::from_events(&all_events)?, deleted);

        let (before_state, _) = before_state.delete()?;
        assert_eq!(before_state.delete().unwrap_err(), Error::AlreadyDeleted);
        Ok(())
    }

    #[test]
    fn test_update() -> anyhow::Result<()> {
        let (before_state, before_events) = build_data_point()?;
        let (updated, events) = before_state.update(YValue::from(456_u32))?;
        assert_eq!(updated.chart_id(), before_state.chart_id());
        assert_eq!(updated.deleted_at(), before_state.deleted_at());
        assert_eq!(updated.id(), before_state.id());
        assert_eq!(updated.x_value(), before_state.x_value());
        assert_eq!(updated.y_value(), YValue::from(456_u32));
        let all_events = {
            let mut e = before_events.clone();
            e.extend(events);
            e
        };
        assert_eq!(DataPoint::from_events(&all_events)?, updated);

        let (before_state, _) = before_state.delete()?;
        assert_eq!(
            before_state.update(YValue::from(456_u32)).unwrap_err(),
            Error::AlreadyDeleted
        );
        Ok(())
    }

    fn build_data_point() -> anyhow::Result<(DataPoint, Vec<DataPointEvent>)> {
        let chart_id = ChartId::generate();
        Ok(DataPoint::create(
            chart_id,
            XValue::from_str("2020-01-02")?,
            YValue::from(123_u32),
        )?)
    }
}
