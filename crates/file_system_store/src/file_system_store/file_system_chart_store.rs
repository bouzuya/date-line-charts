use std::{
    collections::BTreeMap,
    fs::{File, OpenOptions},
    io::{BufRead, BufReader, Write},
    path::PathBuf,
    sync::Arc,
};

use tokio::sync::Mutex;
use write_model::{
    aggregate::Chart,
    event::{BaseEvent, ChartCreated, ChartDeleted, ChartEvent, ChartEventData, ChartUpdated},
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

impl From<&ChartEvent> for EventJson {
    fn from(
        BaseEvent {
            at,
            data,
            id,
            stream_id,
            version,
        }: &ChartEvent,
    ) -> Self {
        Self {
            at: at.to_string(),
            data: match data {
                ChartEventData::Created(ChartCreated { title }) => {
                    EventJsonData::Created(EventJsonDataCreated {
                        title: title.to_owned(),
                    })
                }
                ChartEventData::Deleted(ChartDeleted {}) => {
                    EventJsonData::Deleted(EventJsonDataDeleted {})
                }
                ChartEventData::Updated(ChartUpdated { title }) => {
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

impl TryFrom<EventJson> for ChartEvent {
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
                ChartEventData::Created(ChartCreated { title })
            }
            EventJsonData::Deleted(_) => ChartEventData::Deleted(ChartDeleted {}),
            EventJsonData::Updated(EventJsonDataUpdated { title }) => {
                ChartEventData::Updated(ChartUpdated { title })
            }
        };
        Ok(ChartEvent {
            at: at.parse()?,
            data,
            id: id.parse()?,
            stream_id: stream_id.parse()?,
            version: Version::try_from(version)?,
        })
    }
}

struct Cache {
    command_data: BTreeMap<ChartId, Vec<ChartEvent>>,
    query_data: Vec<query_use_case::port::ChartQueryData>,
}

pub struct FileSystemChartStore {
    cache: Arc<Mutex<Option<Cache>>>,
    dir: PathBuf,
}

impl FileSystemChartStore {
    pub fn new(dir: PathBuf) -> Self {
        Self {
            cache: Arc::new(Mutex::new(None)),
            dir,
        }
    }

    async fn find_impl(
        &self,
        id: ChartId,
    ) -> Result<Option<Chart>, Box<dyn std::error::Error + Send + Sync>> {
        let mut cache = self.cache.lock().await;
        if cache.is_none() {
            *cache = Some(self.load()?);
        }
        Ok(
            match cache
                .as_ref()
                .expect("cache to be Some")
                .command_data
                .get(&id)
            {
                None => None,
                Some(events) => Some(Chart::from_events(events)?),
            },
        )
    }

    async fn get_impl(
        &self,
        id: ChartId,
    ) -> Result<
        Option<query_use_case::port::ChartQueryData>,
        Box<dyn std::error::Error + Send + Sync>,
    > {
        let mut cache = self.cache.lock().await;
        if cache.is_none() {
            *cache = Some(self.load()?);
        }
        Ok(cache
            .as_ref()
            .expect("cache to be Some")
            .query_data
            .iter()
            .find(|chart| chart.id == id)
            .cloned())
    }

    async fn list_impl(
        &self,
    ) -> Result<Vec<query_use_case::port::ChartQueryData>, Box<dyn std::error::Error + Send + Sync>>
    {
        let mut cache = self.cache.lock().await;
        if cache.is_none() {
            *cache = Some(self.load()?);
        }
        Ok(cache.as_ref().expect("cache to be Some").query_data.clone())
    }

    fn load(&self) -> Result<Cache, Box<dyn std::error::Error + Send + Sync>> {
        let path_buf = self.dir.join("charts.jsonl");
        if !path_buf.exists() {
            return Ok(Cache {
                command_data: BTreeMap::new(),
                query_data: Vec::new(),
            });
        }
        let file = File::open(path_buf)?;
        let mut reader = BufReader::new(file);
        let mut command_data = BTreeMap::new();
        let mut query_data = Vec::new();
        let mut buf = String::new();
        while let Ok(size) = reader.read_line(&mut buf) {
            if size == 0 {
                break;
            }
            let event_json = serde_json::from_str::<EventJson>(&buf)?;
            let event = ChartEvent::try_from(event_json)?;
            buf.clear();
            Self::apply_event_to_query_data(&mut query_data, &event)?;
            command_data
                .entry(event.stream_id)
                .or_insert_with(Vec::new)
                .push(event);
        }

        Ok(Cache {
            command_data,
            query_data,
        })
    }

    async fn store_impl(
        &self,
        current: Option<Version>,
        events: &[ChartEvent],
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut cache = self.cache.lock().await;
        if cache.is_none() {
            *cache = Some(self.load()?);
        }
        let cache = cache.as_mut().expect("cache to be Some");
        if events.is_empty() {
            return Ok(());
        }
        match current {
            None => {
                let id = events[0].stream_id;
                cache.command_data.insert(id, events.to_vec());
            }
            Some(_version) => {
                let id = events[0].stream_id;
                let stored_events = cache.command_data.get_mut(&id).ok_or("not found")?;
                // TODO: check version
                stored_events.extend(events.to_vec());
            }
        }
        let mut data = events
            .iter()
            .map(|event| serde_json::to_string(&EventJson::from(event)))
            .collect::<serde_json::Result<Vec<String>>>()?
            .join("\n");
        data.push('\n');
        let path_buf = self.dir.join("charts.jsonl");
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(path_buf)?;
        file.write_all(data.as_bytes())?;

        // query writer
        let query_data = &mut cache.query_data;
        for event in events {
            Self::apply_event_to_query_data(query_data, event)?;
        }

        Ok(())
    }

    fn apply_event_to_query_data(
        query_data: &mut Vec<query_use_case::port::ChartQueryData>,
        event: &ChartEvent,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        match &event.data {
            write_model::event::ChartEventData::Created(data) => {
                query_data.push(query_use_case::port::ChartQueryData {
                    created_at: event.at,
                    id: event.stream_id,
                    title: data.title.clone(),
                });
            }
            write_model::event::ChartEventData::Deleted(_) => {
                if let Some(index) = query_data
                    .iter()
                    .position(|chart| chart.id == event.stream_id)
                {
                    query_data.remove(index);
                }
            }
            write_model::event::ChartEventData::Updated(data) => {
                let index = query_data
                    .iter()
                    .position(|chart| chart.id == event.stream_id)
                    .ok_or("not found")?;
                query_data[index].title.clone_from(&data.title);
            }
        }
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
        events: &[ChartEvent],
    ) -> Result<(), command_use_case::port::chart_repository::Error> {
        self.store_impl(current, events)
            .await
            .map_err(command_use_case::port::chart_repository::Error::from)
    }
}

#[async_trait::async_trait]
impl query_use_case::port::ChartReader for FileSystemChartStore {
    async fn get(
        &self,
        id: ChartId,
    ) -> Result<
        Option<query_use_case::port::ChartQueryData>,
        query_use_case::port::chart_reader::Error,
    > {
        self.get_impl(id)
            .await
            .map_err(query_use_case::port::chart_reader::Error::from)
    }

    async fn list(
        &self,
    ) -> Result<Vec<query_use_case::port::ChartQueryData>, query_use_case::port::chart_reader::Error>
    {
        self.list_impl()
            .await
            .map_err(query_use_case::port::chart_reader::Error::from)
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
