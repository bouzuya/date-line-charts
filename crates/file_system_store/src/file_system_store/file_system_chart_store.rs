use std::{
    collections::BTreeMap,
    fs::File,
    io::{BufRead, BufReader},
    path::PathBuf,
    sync::Arc,
};

use tokio::sync::Mutex;
use write_model::{
    aggregate::{
        chart::{
            event::{Created, Deleted, Updated},
            Event, EventData,
        },
        Chart,
    },
    value_object::{ChartId, Version},
};

#[derive(Debug, serde::Deserialize)]
struct EventJson {
    at: String,
    data: EventJsonData,
    id: String,
    stream_id: String,
    version: i64,
}

#[derive(Debug, serde::Deserialize)]
#[serde(tag = "type")]
pub enum EventJsonData {
    Created(EventJsonDataCreated),
    Deleted(EventJsonDataDeleted),
    Updated(EventJsonDataUpdated),
}

#[derive(Debug, serde::Deserialize)]
struct EventJsonDataCreated {
    title: String,
}

#[derive(Debug, serde::Deserialize)]
struct EventJsonDataDeleted {}

#[derive(Debug, serde::Deserialize)]
struct EventJsonDataUpdated {
    title: String,
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

pub struct FileSystemChartStore {
    command_data: Arc<Mutex<Option<BTreeMap<ChartId, Vec<Event>>>>>,
    dir: PathBuf,
}

impl FileSystemChartStore {
    pub fn new(dir: PathBuf) -> Self {
        Self {
            command_data: Arc::new(Mutex::new(None)),
            dir,
        }
    }
}

#[async_trait::async_trait]
impl command_use_case::port::ChartRepository for FileSystemChartStore {
    async fn find(
        &self,
        id: ChartId,
    ) -> Result<Option<Chart>, Box<dyn std::error::Error + Send + Sync>> {
        let mut command_data = self.command_data.lock().await;
        if command_data.is_none() {
            let path_buf = self.dir.join("charts.jsonl");
            let mut loaded = BTreeMap::new();
            let file = File::open(path_buf)?;
            let mut reader = BufReader::new(file);
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
            *command_data = Some(loaded);
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

    async fn store(
        &self,
        _current: Option<Version>,
        _events: &[Event],
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        todo!()
    }
}
