use std::{collections::BTreeMap, sync::Arc};

use command_use_case::port::ChartRepository;
use query_use_case::port::ChartQueryData;
use tokio::sync::Mutex;
use write_model::{
    aggregate::{chart::Event, Chart},
    value_object::{ChartId, Version},
};

pub(crate) struct ChartDatabase {
    command_data: Arc<Mutex<BTreeMap<ChartId, Vec<Event>>>>,
    query_data: Arc<Mutex<Vec<ChartQueryData>>>,
}

impl ChartDatabase {
    pub(crate) fn new() -> Self {
        Self {
            command_data: Arc::new(Mutex::new(BTreeMap::new())),
            query_data: Arc::new(Mutex::new(Vec::new())),
        }
    }
}

#[async_trait::async_trait]
impl query_use_case::port::ChartReader for ChartDatabase {
    async fn get(
        &self,
        id: ChartId,
    ) -> Result<ChartQueryData, Box<dyn std::error::Error + Send + Sync>> {
        let query_data = self.query_data.lock().await;
        Ok(query_data
            .iter()
            .find(|chart| chart.id == id)
            .cloned()
            .ok_or(query_use_case::get_chart::Error)?)
    }

    async fn list(&self) -> Result<Vec<ChartQueryData>, Box<dyn std::error::Error + Send + Sync>> {
        let query_data = self.query_data.lock().await;
        Ok(query_data.iter().cloned().collect::<Vec<ChartQueryData>>())
    }
}

#[async_trait::async_trait]
impl ChartRepository for ChartDatabase {
    async fn find(
        &self,
        id: ChartId,
    ) -> Result<Option<Chart>, Box<dyn std::error::Error + Send + Sync>> {
        let command_data = self.command_data.lock().await;
        Ok(match command_data.get(&id) {
            None => None,
            Some(events) => Some(Chart::from_events(events)?),
        })
    }

    async fn store(
        &self,
        current: Option<Version>,
        events: &[Event],
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut command_data = self.command_data.lock().await;
        let mut query_data = self.query_data.lock().await;
        if events.is_empty() {
            return Ok(());
        }
        match current {
            None => {
                let id = events[0].id;
                command_data.insert(id, events.to_vec());

                let state = Chart::from_events(events)?;
                query_data.push(ChartQueryData {
                    created_at: state.created_at(),
                    id: state.id(),
                    title: state.title().to_string(),
                });
            }
            Some(_version) => {
                let id = events[0].id;
                let stored_events = command_data.get_mut(&id).ok_or("not found")?;
                // TODO: check version
                stored_events.extend(events.to_vec());

                let state = Chart::from_events(stored_events)?;
                let index = query_data
                    .iter()
                    .position(|chart| chart.id == state.id())
                    .ok_or("not found")?;
                if state.deleted_at().is_some() {
                    query_data.remove(index);
                } else {
                    query_data[index] = ChartQueryData {
                        created_at: state.created_at(),
                        id: state.id(),
                        title: state.title().to_string(),
                    };
                }
            }
        }

        Ok(())
    }
}
