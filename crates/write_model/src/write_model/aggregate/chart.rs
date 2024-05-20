pub mod event;

use crate::value_object::{ChartId, DateTime, Version};

pub use self::event::Event;
use self::event::{Created, Deleted, EventData, Updated};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("created not found")]
    CreatedNotFound,
    #[error("invalid title")]
    InvalidTitle,
    #[error("multiple created")]
    MultipleCreated,
    #[error("overflow version")]
    OverflowVersion,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Chart {
    created_at: DateTime,
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
        let at = DateTime::now();
        let id = ChartId::generate();
        let version = Version::new();
        let events = vec![Event {
            at,
            data: EventData::Created(Created {
                title: title.clone(),
            }),
            id,
            version,
        }];
        let state = Self {
            created_at: at,
            deleted_at: None,
            id,
            title,
            version,
        };
        Ok((state, events))
    }

    pub fn from_events(events: &[Event]) -> Result<Self, Error> {
        let mut state = match events.first() {
            None => return Err(Error::CreatedNotFound),
            Some(Event {
                at,
                data: EventData::Created(event),
                id,
                version,
            }) => Self {
                created_at: *at,
                deleted_at: None,
                id: *id,
                title: event.title.clone(),
                version: *version,
            },
            Some(_) => return Err(Error::CreatedNotFound),
        };
        state.apply_events(&events[1..])?;
        Ok(state)
    }

    pub fn reconstruct(
        created_at: DateTime,
        deleted_at: Option<DateTime>,
        id: ChartId,
        title: String,
        version: Version,
    ) -> Self {
        Self {
            created_at,
            deleted_at,
            id,
            title,
            version,
        }
    }

    pub fn created_at(&self) -> DateTime {
        self.created_at
    }

    pub fn delete(&self) -> Result<(Self, Vec<Event>), Error> {
        let at = DateTime::now();
        let version = self.version.next().map_err(|_| Error::OverflowVersion)?;
        let events = vec![Event {
            at,
            data: EventData::Deleted(Deleted {}),
            id: self.id,
            version,
        }];
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
        if title.is_empty() {
            return Err(Error::InvalidTitle);
        }
        let at = DateTime::now();
        let version = self.version.next().map_err(|_| Error::OverflowVersion)?;
        let events = vec![Event {
            at,
            data: EventData::Updated(Updated {
                title: title.clone(),
            }),
            id: self.id,
            version,
        }];
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
                EventData::Created(_) => return Err(Error::MultipleCreated),
                EventData::Updated(e) => {
                    self.title = e.title.clone();
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
}
