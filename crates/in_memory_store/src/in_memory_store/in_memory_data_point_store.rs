use std::{collections::BTreeMap, sync::Arc};

use tokio::sync::Mutex;
use write_model::{
    aggregate::{data_point::Event, DataPoint},
    value_object::{ChartId, DataPointId, Version},
};

pub struct InMemoryDataPointStore {
    command_data: Arc<Mutex<BTreeMap<DataPointId, Vec<Event>>>>,
    query_data: Arc<Mutex<Vec<query_use_case::port::DataPointQueryData>>>,
}

impl InMemoryDataPointStore {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self {
            command_data: Arc::new(Mutex::new(BTreeMap::new())),
            query_data: Arc::new(Mutex::new(Vec::new())),
        }
    }

    async fn find_impl(
        &self,
        id: DataPointId,
    ) -> Result<Option<DataPoint>, Box<dyn std::error::Error + Send + Sync>> {
        let command_data = self.command_data.lock().await;
        Ok(match command_data.get(&id) {
            None => None,
            Some(events) => Some(DataPoint::from_events(events)?),
        })
    }

    async fn get_impl(
        &self,
        id: DataPointId,
    ) -> Result<
        Option<query_use_case::port::DataPointQueryData>,
        Box<dyn std::error::Error + Send + Sync>,
    > {
        let query_data = self.query_data.lock().await;
        Ok(query_data
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
        let query_data = self.query_data.lock().await;
        Ok(query_data
            .iter()
            .filter(|data_point| data_point.chart_id == chart_id)
            .cloned()
            .collect::<Vec<query_use_case::port::DataPointQueryData>>())
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
                write_model::aggregate::data_point::EventData::Created(data) => {
                    query_data.push(query_use_case::port::DataPointQueryData {
                        created_at: event.at,
                        chart_id: event.stream_id.chart_id(),
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
        }

        Ok(())
    }
}

#[async_trait::async_trait]
impl query_use_case::port::DataPointReader for InMemoryDataPointStore {
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

#[async_trait::async_trait]
impl command_use_case::port::DataPointRepository for InMemoryDataPointStore {
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
