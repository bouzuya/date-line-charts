pub mod event;

use crate::value_object::{ChartId, DateTime, Version};

use self::event::{Created, Deleted, Updated};
pub use self::event::{Event, EventData};

#[derive(Debug, PartialEq, thiserror::Error)]
pub enum Error {
    #[error("already deleted")]
    AlreadyDeleted,
    #[error("invalid title")]
    InvalidTitle,
    #[error("multiple created event")]
    MultipleCreatedEvent,
    #[error("no created event")]
    NoCreatedEvent,
    #[error("version overflow")]
    VersionOverflow,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Chart {
    deleted_at: Option<DateTime>,
    id: ChartId,
    title: String,
    version: Version,
}

impl Chart {
    pub fn create(title: String) -> Result<(Self, Vec<Event>), Error> {
        if title.is_empty() {
            return Err(Error::InvalidTitle);
        }
        let events = vec![Event::new(
            ChartId::generate(),
            EventData::Created(Created {
                title: title.clone(),
            }),
            Version::new(),
        )];
        let state = Self {
            deleted_at: None,
            id: events[0].stream_id,
            title,
            version: events[0].version,
        };
        Ok((state, events))
    }

    pub fn from_events(events: &[Event]) -> Result<Self, Error> {
        let mut state = match events.first() {
            None => return Err(Error::NoCreatedEvent),
            Some(Event {
                at: _,
                data: EventData::Created(event),
                id: _,
                stream_id,
                version,
            }) => Self {
                deleted_at: None,
                id: *stream_id,
                title: event.title.clone(),
                version: *version,
            },
            Some(_) => return Err(Error::NoCreatedEvent),
        };
        state.apply_events(&events[1..])?;
        Ok(state)
    }

    pub fn reconstruct(
        deleted_at: Option<DateTime>,
        id: ChartId,
        title: String,
        version: Version,
    ) -> Self {
        Self {
            deleted_at,
            id,
            title,
            version,
        }
    }

    pub fn delete(&self) -> Result<(Self, Vec<Event>), Error> {
        if self.deleted_at.is_some() {
            return Err(Error::AlreadyDeleted);
        }
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

    pub fn id(&self) -> ChartId {
        self.id
    }

    pub fn title(&self) -> &str {
        &self.title
    }

    pub fn update(&self, title: String) -> Result<(Self, Vec<Event>), Error> {
        if self.deleted_at.is_some() {
            return Err(Error::AlreadyDeleted);
        }
        if title.is_empty() {
            return Err(Error::InvalidTitle);
        }
        let events = vec![Event::new(
            self.id,
            EventData::Updated(Updated {
                title: title.clone(),
            }),
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
                    self.title.clone_from(&e.title);
                    self.version = version;
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
    use super::*;

    #[test]
    fn test() -> anyhow::Result<()> {
        let mut all_events = vec![];
        let (state, events) = Chart::create("title1".to_string())?;
        all_events.extend(events);
        assert_eq!(Chart::from_events(&all_events)?, state);
        assert!(state.deleted_at.is_none());
        assert_eq!(state.title(), "title1");
        let (state, events) = state.update("title2".to_string())?;
        all_events.extend(events);
        assert_eq!(state.title(), "title2");
        assert_eq!(Chart::from_events(&all_events)?, state);
        let (state, events) = state.delete()?;
        all_events.extend(events);
        assert_eq!(Chart::from_events(&all_events)?, state);
        assert!(state.deleted_at.is_some());
        Ok(())
    }

    #[test]
    fn test_delete() -> anyhow::Result<()> {
        let (before_state, before_events) = build_chart()?;
        let (deleted, events) = before_state.delete()?;
        assert!(deleted.deleted_at().is_some());
        assert_eq!(deleted.id(), before_state.id());
        assert_eq!(deleted.title(), before_state.title());
        let all_events = {
            let mut all_events = before_events.clone();
            all_events.extend(events);
            all_events
        };
        assert_eq!(Chart::from_events(&all_events)?, deleted);

        let (before_state, _) = before_state.delete()?;
        assert_eq!(before_state.delete().unwrap_err(), Error::AlreadyDeleted);
        Ok(())
    }

    #[test]
    fn test_update() -> anyhow::Result<()> {
        let (before_state, before_events) = build_chart()?;
        let (updated, events) = before_state.update("title2".to_string())?;
        assert_eq!(updated.deleted_at(), before_state.deleted_at());
        assert_eq!(updated.id(), before_state.id());
        assert_eq!(updated.title(), "title2");
        let all_events = {
            let mut all_events = before_events.clone();
            all_events.extend(events);
            all_events
        };
        assert_eq!(Chart::from_events(&all_events)?, updated);

        let (before_state, _) = before_state.delete()?;
        assert_eq!(
            before_state.update("title2".to_string()).unwrap_err(),
            Error::AlreadyDeleted
        );
        Ok(())
    }

    fn build_chart() -> anyhow::Result<(Chart, Vec<Event>)> {
        Ok(Chart::create("title".to_string())?)
    }
}
