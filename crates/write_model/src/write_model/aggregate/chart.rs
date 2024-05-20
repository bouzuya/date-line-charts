use crate::value_object::{ChartId, DateTime, Version};

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
        let events = vec![Event::Created(Created {
            at,
            id,
            title: title.clone(),
            version,
        })];
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
            Some(Event::Created(event)) => Self {
                created_at: event.at,
                deleted_at: None,
                id: event.id,
                title: event.title.clone(),
                version: event.version,
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
        let events = vec![Event::Deleted(Deleted {
            at,
            id: self.id,
            version,
        })];
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
        let events = vec![Event::Updated(Updated {
            at,
            id: self.id,
            title,
            version,
        })];
        let mut state = self.clone();
        state.apply_events(&events)?;
        Ok((state, events))
    }

    pub fn version(&self) -> Version {
        self.version
    }

    fn apply_events(&mut self, events: &[Event]) -> Result<(), Error> {
        for event in events {
            match event {
                Event::Created(_) => return Err(Error::MultipleCreated),
                Event::Updated(event) => {
                    self.title = event.title.clone();
                    self.version = event.version;
                }
                Event::Deleted(event) => {
                    self.deleted_at = Some(event.at);
                    self.version = event.version;
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
