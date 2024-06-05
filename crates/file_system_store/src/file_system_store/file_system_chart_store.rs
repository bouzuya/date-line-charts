use std::{
    collections::BTreeMap,
    fs::{File, OpenOptions},
    io::{BufRead, BufReader, Write},
    path::PathBuf,
    sync::Arc,
};

use tokio::sync::Mutex;
use write_model::{
    aggregate::{
        chart::{
            event::{BaseEvent, Created, Deleted, Updated},
            Event, EventData,
        },
        Chart,
    },
    value_object::{ChartId, Version},
};

#[derive(Debug, serde::Deserialize, serde::Serialize)]
struct EventJson {
    at: String,
    data: EventJsonData,
    id: String,
    stream_id: String,
    version: i64,
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
#[serde(tag = "type")]
enum EventJsonData {
    Created(EventJsonDataCreated),
    Deleted(EventJsonDataDeleted),
    Updated(EventJsonDataUpdated),
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
struct EventJsonDataCreated {
    title: String,
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
struct EventJsonDataDeleted {}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
struct EventJsonDataUpdated {
    title: String,
}

impl From<&Event> for EventJson {
    fn from(
        BaseEvent {
            at,
            data,
            id,
            stream_id,
            version,
        }: &Event,
    ) -> Self {
        Self {
            at: at.to_string(),
            data: match data {
                EventData::Created(Created { title }) => {
                    EventJsonData::Created(EventJsonDataCreated {
                        title: title.to_owned(),
                    })
                }
                EventData::Deleted(Deleted {}) => EventJsonData::Deleted(EventJsonDataDeleted {}),
                EventData::Updated(Updated { title }) => {
                    EventJsonData::Updated(EventJsonDataUpdated {
                        title: title.to_owned(),
                    })
                }
            },
            id: id.to_string(),
            stream_id: stream_id.to_string(),
            version: i64::from(*version),
        }
    }
}

impl TryFrom<EventJson> for Event {
    type Error = Box<dyn std::error::Error + Send + Sync>;

    fn try_from(
        EventJson {
            at,
            data,
            id,
            stream_id,
            version,
        }: EventJson,
    ) -> Result<Self, Self::Error> {
        let data = match data {
            EventJsonData::Created(EventJsonDataCreated { title }) => {
                EventData::Created(Created { title })
            }
            EventJsonData::Deleted(_) => EventData::Deleted(Deleted {}),
            EventJsonData::Updated(EventJsonDataUpdated { title }) => {
                EventData::Updated(Updated { title })
            }
        };
        Ok(Event {
            at: at.parse()?,
            data,
            id: id.parse()?,
            stream_id: stream_id.parse()?,
            version: Version::try_from(version)?,
        })
    }
}

type CommandData = BTreeMap<ChartId, Vec<Event>>;

pub struct FileSystemChartStore {
    command_data: Arc<Mutex<Option<CommandData>>>,
    dir: PathBuf,
}

impl FileSystemChartStore {
    pub fn new(dir: PathBuf) -> Self {
        Self {
            command_data: Arc::new(Mutex::new(None)),
            dir,
        }
    }

    async fn find_impl(
        &self,
        id: ChartId,
    ) -> Result<Option<Chart>, Box<dyn std::error::Error + Send + Sync>> {
        let mut command_data = self.command_data.lock().await;
        if command_data.is_none() {
            *command_data = Some(self.load()?);
        }
        Ok(
            match command_data
                .as_ref()
                .expect("command_data to be Some")
                .get(&id)
            {
                None => None,
                Some(events) => Some(Chart::from_events(events)?),
            },
        )
    }

    fn load(&self) -> Result<CommandData, Box<dyn std::error::Error + Send + Sync>> {
        let path_buf = self.dir.join("charts.jsonl");
        if !path_buf.exists() {
            return Ok(BTreeMap::new());
        }
        let file = File::open(path_buf)?;
        let mut reader = BufReader::new(file);
        let mut loaded = BTreeMap::new();
        let mut buf = String::new();
        while let Ok(size) = reader.read_line(&mut buf) {
            if size == 0 {
                break;
            }
            let event_json = serde_json::from_str::<EventJson>(&buf)?;
            let event = Event::try_from(event_json)?;
            buf.clear();
            loaded
                .entry(event.stream_id)
                .or_insert_with(Vec::new)
                .push(event);
        }
        Ok(loaded)
    }

    async fn store_impl(
        &self,
        current: Option<Version>,
        events: &[Event],
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut command_data = self.command_data.lock().await;
        if command_data.is_none() {
            *command_data = Some(self.load()?);
        }
        let command_data = command_data.as_mut().expect("command_data to be Some");
        if events.is_empty() {
            return Ok(());
        }
        match current {
            None => {
                let id = events[0].stream_id;
                command_data.insert(id, events.to_vec());
            }
            Some(_version) => {
                let id = events[0].stream_id;
                let stored_events = command_data.get_mut(&id).ok_or("not found")?;
                // TODO: check version
                stored_events.extend(events.to_vec());
            }
        }
        let data = events
            .iter()
            .map(|event| serde_json::to_string(&EventJson::from(event)))
            .collect::<serde_json::Result<Vec<String>>>()?
            .join("\n");
        let path_buf = self.dir.join("charts.jsonl");
        let mut file = OpenOptions::new()
            .create_new(true)
            .append(true)
            .open(path_buf)?;
        file.write_all(data.as_bytes())?;
        Ok(())
    }
}

#[async_trait::async_trait]
impl command_use_case::port::ChartRepository for FileSystemChartStore {
    async fn find(
        &self,
        id: ChartId,
    ) -> Result<Option<Chart>, command_use_case::port::chart_repository::Error> {
        self.find_impl(id)
            .await
            .map_err(command_use_case::port::chart_repository::Error::from)
    }

    async fn store(
        &self,
        current: Option<Version>,
        events: &[Event],
    ) -> Result<(), command_use_case::port::chart_repository::Error> {
        self.store_impl(current, events)
            .await
            .map_err(command_use_case::port::chart_repository::Error::from)
    }
}

#[cfg(test)]
mod tests {
    use command_use_case::port::ChartRepository;
    use tempdir::TempDir;

    use super::*;

    #[tokio::test]
    async fn test() -> anyhow::Result<()> {
        let temp_dir = TempDir::new("file_system_store")?;
        let path_buf = temp_dir.into_path();
        let store = FileSystemChartStore::new(path_buf.clone());
        let (state, events) = Chart::create("title1".to_string())?;
        let chart_id = state.id();
        assert!(store.find(chart_id).await?.is_none());
        store.store(None, &events).await?;
        assert_eq!(store.find(chart_id).await?, Some(state.clone()));

        let store = FileSystemChartStore::new(path_buf.clone());
        assert_eq!(store.find(chart_id).await?, Some(state));
        Ok(())
    }
}
