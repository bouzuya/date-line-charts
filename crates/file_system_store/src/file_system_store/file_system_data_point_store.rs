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
        chart::event::BaseEvent,
        data_point::{
            event::{Created, Deleted, Updated},
            Event, EventData,
        },
        DataPoint,
    },
    value_object::{ChartId, DataPointId, Version, YValue},
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
    value: u32,
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
struct EventJsonDataDeleted {}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
struct EventJsonDataUpdated {
    value: u32,
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
                EventData::Created(Created { value }) => {
                    EventJsonData::Created(EventJsonDataCreated {
                        value: u32::from(*value),
                    })
                }
                EventData::Deleted(Deleted {}) => EventJsonData::Deleted(EventJsonDataDeleted {}),
                EventData::Updated(Updated { value }) => {
                    EventJsonData::Updated(EventJsonDataUpdated {
                        value: u32::from(*value),
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
            EventJsonData::Created(EventJsonDataCreated { value }) => EventData::Created(Created {
                value: YValue::from(value),
            }),
            EventJsonData::Deleted(_) => EventData::Deleted(Deleted {}),
            EventJsonData::Updated(EventJsonDataUpdated { value }) => EventData::Updated(Updated {
                value: YValue::from(value),
            }),
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

struct Cache {
    command_data: BTreeMap<DataPointId, Vec<Event>>,
    query_data: Vec<query_use_case::port::DataPointQueryData>,
}

pub struct FileSystemDataPointStore {
    cache: Arc<Mutex<Option<Cache>>>,
    dir: PathBuf,
}

impl FileSystemDataPointStore {
    pub fn new(dir: PathBuf) -> Self {
        Self {
            cache: Arc::new(Mutex::new(None)),
            dir,
        }
    }

    async fn find_impl(
        &self,
        id: DataPointId,
    ) -> Result<Option<DataPoint>, Box<dyn std::error::Error + Send + Sync>> {
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
                Some(events) => Some(DataPoint::from_events(events)?),
            },
        )
    }

    async fn get_impl(
        &self,
        id: DataPointId,
    ) -> Result<
        Option<query_use_case::port::DataPointQueryData>,
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
            .find(|data_point| {
                data_point.chart_id == id.chart_id() && data_point.x_value == id.x_value()
            })
            .cloned())
    }

    async fn list_impl(
        &self,
        chart_id: ChartId,
    ) -> Result<
        Vec<query_use_case::port::DataPointQueryData>,
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
            .filter(|data_point| data_point.chart_id == chart_id)
            .cloned()
            .collect::<Vec<query_use_case::port::DataPointQueryData>>())
    }

    fn load(&self) -> Result<Cache, Box<dyn std::error::Error + Send + Sync>> {
        let path_buf = self.dir.join("data_points.jsonl");
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
            let event = Event::try_from(event_json)?;
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
        events: &[Event],
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
        let path_buf = self.dir.join("data_points.jsonl");
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
        query_data: &mut Vec<query_use_case::port::DataPointQueryData>,
        event: &Event,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        match &event.data {
            write_model::aggregate::data_point::EventData::Created(data) => {
                query_data.push(query_use_case::port::DataPointQueryData {
                    chart_id: event.stream_id.chart_id(),
                    created_at: event.at,
                    x_value: event.stream_id.x_value(),
                    y_value: data.value,
                });
            }
            write_model::aggregate::data_point::EventData::Deleted(_) => {
                if let Some(index) = query_data.iter().position(|data_point| {
                    data_point.chart_id == event.stream_id.chart_id()
                        && data_point.x_value == event.stream_id.x_value()
                }) {
                    query_data.remove(index);
                }
            }
            write_model::aggregate::data_point::EventData::Updated(data) => {
                let index = query_data
                    .iter()
                    .position(|data_point| {
                        data_point.chart_id == event.stream_id.chart_id()
                            && data_point.x_value == event.stream_id.x_value()
                    })
                    .ok_or("not found")?;
                query_data[index].y_value = data.value;
            }
        }
        Ok(())
    }
}

#[async_trait::async_trait]
impl command_use_case::port::DataPointRepository for FileSystemDataPointStore {
    async fn find(
        &self,
        id: DataPointId,
    ) -> Result<Option<DataPoint>, command_use_case::port::data_point_repository::Error> {
        self.find_impl(id)
            .await
            .map_err(command_use_case::port::data_point_repository::Error::from)
    }

    async fn store(
        &self,
        current: Option<Version>,
        events: &[Event],
    ) -> Result<(), command_use_case::port::data_point_repository::Error> {
        self.store_impl(current, events)
            .await
            .map_err(command_use_case::port::data_point_repository::Error::from)
    }
}

#[async_trait::async_trait]
impl query_use_case::port::DataPointReader for FileSystemDataPointStore {
    async fn get(
        &self,
        id: DataPointId,
    ) -> Result<
        Option<query_use_case::port::DataPointQueryData>,
        query_use_case::port::data_point_reader::Error,
    > {
        self.get_impl(id)
            .await
            .map_err(query_use_case::port::data_point_reader::Error::from)
    }

    async fn list(
        &self,
        chart_id: ChartId,
    ) -> Result<
        Vec<query_use_case::port::DataPointQueryData>,
        query_use_case::port::data_point_reader::Error,
    > {
        self.list_impl(chart_id)
            .await
            .map_err(query_use_case::port::data_point_reader::Error::from)
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use command_use_case::port::DataPointRepository;
    use tempdir::TempDir;
    use write_model::value_object::XValue;

    use super::*;

    #[tokio::test]
    async fn test() -> anyhow::Result<()> {
        let temp_dir = TempDir::new("file_system_store")?;
        let path_buf = temp_dir.into_path();
        let store = FileSystemDataPointStore::new(path_buf.clone());
        let chart_id = ChartId::generate();
        let (state, events) = DataPoint::create(
            chart_id,
            XValue::from_str("2020-01-02")?,
            YValue::from(123_u32),
        )?;
        let data_point_id = state.id();
        assert!(store.find(data_point_id).await?.is_none());
        store.store(None, &events).await?;
        assert_eq!(store.find(data_point_id).await?, Some(state.clone()));

        let store = FileSystemDataPointStore::new(path_buf.clone());
        assert_eq!(store.find(data_point_id).await?, Some(state));
        Ok(())
    }
}
