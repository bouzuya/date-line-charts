use std::{collections::BTreeMap, sync::Arc};

use tokio::sync::Mutex;
use write_model::{
    aggregate::{chart::Event, Chart},
    value_object::{ChartId, Version},
};

pub struct InMemoryChartStore {
    command_data: Arc<Mutex<BTreeMap<ChartId, Vec<Event>>>>,
    query_data: Arc<Mutex<Vec<query_use_case::port::ChartQueryData>>>,
}

impl InMemoryChartStore {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self {
            command_data: Arc::new(Mutex::new(BTreeMap::new())),
            query_data: Arc::new(Mutex::new(Vec::new())),
        }
    }

    async fn find_impl(
        &self,
        id: ChartId,
    ) -> Result<Option<Chart>, Box<dyn std::error::Error + Send + Sync>> {
        let command_data = self.command_data.lock().await;
        Ok(match command_data.get(&id) {
            None => None,
            Some(events) => Some(Chart::from_events(events)?),
        })
    }

    async fn get_impl(
        &self,
        id: ChartId,
    ) -> Result<
        Option<query_use_case::port::ChartQueryData>,
        Box<dyn std::error::Error + Send + Sync>,
    > {
        let query_data = self.query_data.lock().await;
        Ok(query_data.iter().find(|chart| chart.id == id).cloned())
    }

    async fn store_impl(
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

        // query writer
        for event in events {
            match &event.data {
                write_model::aggregate::chart::EventData::Created(data) => {
                    query_data.push(query_use_case::port::ChartQueryData {
                        created_at: event.at,
                        id: event.stream_id,
                        title: data.title.clone(),
                    });
                }
                write_model::aggregate::chart::EventData::Deleted(_) => {
                    if let Some(index) = query_data
                        .iter()
                        .position(|chart| chart.id == event.stream_id)
                    {
                        query_data.remove(index);
                    }
                }
                write_model::aggregate::chart::EventData::Updated(data) => {
                    let index = query_data
                        .iter()
                        .position(|chart| chart.id == event.stream_id)
                        .ok_or("not found")?;
                    query_data[index].title.clone_from(&data.title);
                }
            }
        }

        Ok(())
    }
}

#[async_trait::async_trait]
impl query_use_case::port::ChartReader for InMemoryChartStore {
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
        let query_data = self.query_data.lock().await;
        Ok(query_data
            .iter()
            .cloned()
            .collect::<Vec<query_use_case::port::ChartQueryData>>())
    }
}

#[async_trait::async_trait]
impl command_use_case::port::ChartRepository for InMemoryChartStore {
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
