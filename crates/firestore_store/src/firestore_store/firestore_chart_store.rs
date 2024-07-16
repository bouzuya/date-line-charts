use std::str::FromStr;

use crate::{
    firestore_event_store::FirestoreEventStore, firestore_query_data_store::FirestoreQueryDataStore,
};
use write_model::{
    aggregate::Chart,
    event::{ChartEvent, Event},
    value_object::{ChartId, EventStreamId, Version},
};

pub struct FirestoreChartStore {
    event_store: FirestoreEventStore,
    query_data_store: FirestoreQueryDataStore,
}

impl FirestoreChartStore {
    pub async fn new() -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        Ok(Self {
            event_store: FirestoreEventStore::new().await?,
            query_data_store: FirestoreQueryDataStore::new().await?,
        })
    }

    async fn reader_get_impl(
        &self,
        id: ChartId,
    ) -> Result<
        Option<query_use_case::port::ChartQueryData>,
        Box<dyn std::error::Error + Send + Sync>,
    > {
        self.query_data_store.get_chart(id).await
    }

    async fn reader_list_impl(
        &self,
    ) -> Result<Vec<query_use_case::port::ChartQueryData>, Box<dyn std::error::Error + Send + Sync>>
    {
        self.query_data_store.list_charts().await
    }

    async fn repository_find_impl(
        &self,
        id: ChartId,
    ) -> Result<Option<Chart>, Box<dyn std::error::Error + Send + Sync>> {
        let event_stream_id = EventStreamId::from_str(id.to_string().as_str())?;
        let events = self
            .event_store
            .find_events_by_event_stream_id(&event_stream_id)
            .await?
            .into_iter()
            .map(|event| match event {
                write_model::event::Event::Chart(event) => event,
                write_model::event::Event::DataPoint(_) => unreachable!(),
            })
            .collect::<Vec<ChartEvent>>();
        Ok(Some(Chart::from_events(&events)?))
    }

    async fn repository_store_impl(
        &self,
        current: Option<Version>,
        events: Vec<ChartEvent>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.event_store
            .store(
                current,
                events.into_iter().map(Event::from).collect::<Vec<Event>>(),
            )
            .await?;

        // To simplify the structure, update the query data at this timing (not supported for failure).
        self.query_data_store.update().await?;

        Ok(())
    }
}

#[async_trait::async_trait]
impl query_use_case::port::ChartReader for FirestoreChartStore {
    async fn get(
        &self,
        id: ChartId,
    ) -> Result<
        Option<query_use_case::port::ChartQueryData>,
        query_use_case::port::chart_reader::Error,
    > {
        self.reader_get_impl(id)
            .await
            .map_err(query_use_case::port::chart_reader::Error::from)
    }

    async fn list(
        &self,
    ) -> Result<Vec<query_use_case::port::ChartQueryData>, query_use_case::port::chart_reader::Error>
    {
        self.reader_list_impl()
            .await
            .map_err(query_use_case::port::chart_reader::Error::from)
    }
}

#[async_trait::async_trait]
impl command_use_case::port::ChartRepository for FirestoreChartStore {
    #[tracing::instrument(level = tracing::Level::DEBUG, err(Debug), ret, skip(self))]
    async fn find(
        &self,
        id: ChartId,
    ) -> Result<Option<Chart>, command_use_case::port::chart_repository::Error> {
        self.repository_find_impl(id)
            .await
            .map_err(command_use_case::port::chart_repository::Error::from)
    }

    #[tracing::instrument(level = tracing::Level::DEBUG, err(Debug), ret, skip(self))]
    async fn store(
        &self,
        current: Option<Version>,
        events: &[ChartEvent],
    ) -> Result<(), command_use_case::port::chart_repository::Error> {
        self.repository_store_impl(current, events.to_vec())
            .await
            .map_err(command_use_case::port::chart_repository::Error::from)
    }
}

#[cfg(test)]
mod tests {
    use command_use_case::port::ChartRepository as _;

    use super::*;

    #[ignore = "requires Firestore"]
    #[tokio::test]
    async fn test() -> anyhow::Result<()> {
        let store = FirestoreChartStore::new()
            .await
            .map_err(|e| anyhow::anyhow!(e))?;
        let (chart, events) = Chart::create("title1".to_owned())?;
        assert_eq!(store.find(chart.id()).await?, None);
        store.store(None, &events).await?;
        assert_eq!(store.find(chart.id()).await?, None);
        Ok(())
    }
}
