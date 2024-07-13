use std::str::FromStr as _;

use crate::{
    converter, firestore_event_store::FirestoreEventStore, path, schema::DataPointDocumentData,
};
use firestore_client::FirestoreClient;
use write_model::{
    aggregate::DataPoint,
    event::{DataPointEvent, Event},
    value_object::{ChartId, DataPointId, EventStreamId, Version},
};

pub struct FirestoreDataPointStore {
    client: FirestoreClient,
    event_store: FirestoreEventStore,
}

impl FirestoreDataPointStore {
    pub async fn new() -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        Ok(Self {
            client: FirestoreClient::new().await?,
            event_store: FirestoreEventStore::new().await?,
        })
    }

    async fn reader_get_impl(
        &self,
        id: DataPointId,
    ) -> Result<
        Option<query_use_case::port::DataPointQueryData>,
        Box<dyn std::error::Error + Send + Sync>,
    > {
        self.client
            .get_document::<DataPointDocumentData>(&path::data_point_document(id))
            .await?
            .map(converter::data_point_query_data_from_document)
            .transpose()
    }

    async fn reader_list_impl(
        &self,
        chart_id: ChartId,
    ) -> Result<
        Vec<query_use_case::port::DataPointQueryData>,
        Box<dyn std::error::Error + Send + Sync>,
    > {
        let documents = self
            .client
            .list_all_documents::<DataPointDocumentData>(&path::data_point_collection(chart_id))
            .await?;
        let documents = documents
            .into_iter()
            .map(converter::data_point_query_data_from_document)
            .collect::<Result<Vec<_>, _>>()?;
        Ok(documents)
    }

    async fn repository_find_impl(
        &self,
        id: DataPointId,
    ) -> Result<Option<DataPoint>, Box<dyn std::error::Error + Send + Sync>> {
        let event_stream_id = EventStreamId::from_str(id.to_string().as_str())?;
        let events = self
            .event_store
            .find_events_by_event_stream_id(&event_stream_id)
            .await?
            .into_iter()
            .map(|event| match event {
                write_model::event::Event::Chart(_) => unreachable!(),
                write_model::event::Event::DataPoint(event) => event,
            })
            .collect::<Vec<DataPointEvent>>();
        Ok(Some(DataPoint::from_events(&events)?))
    }

    async fn repository_store_impl(
        &self,
        current: Option<Version>,
        events: Vec<DataPointEvent>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.event_store
            .store(
                current,
                events.into_iter().map(Event::from).collect::<Vec<Event>>(),
            )
            .await?;

        // FIXME: update query data

        Ok(())
    }
}

#[async_trait::async_trait]
impl command_use_case::port::DataPointRepository for FirestoreDataPointStore {
    async fn find(
        &self,
        id: DataPointId,
    ) -> Result<Option<DataPoint>, command_use_case::port::data_point_repository::Error> {
        self.repository_find_impl(id)
            .await
            .map_err(command_use_case::port::data_point_repository::Error::from)
    }

    async fn store(
        &self,
        current: Option<Version>,
        events: &[DataPointEvent],
    ) -> Result<(), command_use_case::port::data_point_repository::Error> {
        self.repository_store_impl(current, events.to_vec())
            .await
            .map_err(command_use_case::port::data_point_repository::Error::from)
    }
}

#[async_trait::async_trait]
impl query_use_case::port::DataPointReader for FirestoreDataPointStore {
    async fn get(
        &self,
        id: DataPointId,
    ) -> Result<
        Option<query_use_case::port::DataPointQueryData>,
        query_use_case::port::data_point_reader::Error,
    > {
        self.reader_get_impl(id)
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
        self.reader_list_impl(chart_id)
            .await
            .map_err(query_use_case::port::data_point_reader::Error::from)
    }
}
