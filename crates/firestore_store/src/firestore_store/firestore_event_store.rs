use std::{future::Future, pin::Pin};

use crate::{
    converter, path,
    schema::{EventDocumentData, EventStreamDocumentData},
};
use firestore_client::{FieldPath, Filter, FirestoreClient, Transaction};
use write_model::{
    event::Event,
    value_object::{EventStreamId, Version},
};

pub struct FirestoreEventStore(FirestoreClient);

impl FirestoreEventStore {
    pub async fn new() -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        Ok(Self(FirestoreClient::new().await?))
    }

    pub async fn find_events_by_event_stream_id(
        &self,
        event_stream_id: &EventStreamId,
    ) -> Result<Vec<Event>, Box<dyn std::error::Error + Send + Sync>> {
        let event_stream_id = event_stream_id.to_string();
        let event_stream = self
            .0
            .get_document::<EventStreamDocumentData>(&path::event_stream_document(&event_stream_id))
            .await?;
        if event_stream.is_none() {
            return Ok(vec![]);
        }
        let collection_path = path::event_collection();
        let mut start_after = None;
        let mut all_documents = vec![];
        loop {
            let documents = self
                .0
                .run_collection_query::<EventDocumentData>(
                    &collection_path,
                    Some(Filter::and([FieldPath::raw("stream_id")
                        .equal(firestore_client::to_value(&event_stream_id)?)?])),
                    Some([FieldPath::raw("version").ascending()]),
                    start_after.clone(),
                    Some(100),
                )
                .await?;
            let is_end = documents.is_empty() || documents.len() < 100;
            start_after = Some([firestore_client::to_value(
                &documents
                    .last()
                    .expect("documents to have at least one element")
                    .fields
                    .version,
            )?]);
            all_documents.extend(documents);
            if is_end {
                break;
            }
        }
        let events = all_documents
            .into_iter()
            .map(converter::event_from_document)
            .collect::<Result<Vec<Event>, Box<dyn std::error::Error + Send + Sync>>>()?;
        Ok(events)
    }

    pub async fn store(
        &self,
        current: Option<Version>,
        events: Vec<Event>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if events.is_empty() {
            return Ok(());
        }

        self.run_transaction(move |transaction| {
            Box::pin(async move {
                let event_stream_id = events[0].stream_id();
                let last_event = events.last().expect("events to have at least one element");
                let last_event_version = last_event.version();
                let last_event_at = last_event.at();
                match current {
                    None => {
                        // create event_stream
                        transaction.create(
                            &path::event_stream_document(event_stream_id.as_ref()),
                            &EventStreamDocumentData {
                                id: event_stream_id.to_string(),
                                last_event_at: last_event_at.to_string(),
                                version: i64::from(last_event_version),
                            },
                        )?;
                    }
                    Some(current) => {
                        // get event_stream with lock
                        let event_stream = transaction
                            .get::<EventStreamDocumentData>(&path::event_stream_document(
                                event_stream_id.to_string().as_str(),
                            ))
                            .await?
                            .ok_or("event stream not found")?;

                        // check version
                        if event_stream.fields.version != i64::from(current) {
                            return Err("version mismatch".into());
                        }

                        // update event_stream
                        transaction.update(
                            &path::event_stream_document(event_stream_id.to_string().as_str()),
                            &EventStreamDocumentData {
                                last_event_at: last_event_at.to_string(),
                                version: i64::from(last_event_version),
                                ..event_stream.fields
                            },
                        )?;
                    }
                }
                // create events
                for event in events {
                    transaction.create(
                        &path::event_document(event.id()),
                        &converter::event_document_data_from_event(&event),
                    )?;
                }
                Ok(())
            })
        })
        .await
    }

    async fn run_transaction<F>(
        &self,
        callback: F,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>>
    where
        F: FnOnce(
            &mut Transaction,
        ) -> Pin<
            Box<
                dyn Future<Output = Result<(), Box<dyn std::error::Error + Send + Sync>>>
                    + Send
                    + '_,
            >,
        >,
    {
        let mut transaction = self.0.begin_transaction().await?;
        let result = match callback(&mut transaction).await {
            Ok(()) => transaction.commit().await.map_err(Into::into),
            Err(e) => Err(e),
        };
        match result {
            Ok(()) => Ok(()),
            Err(e) => {
                // ignore rollback error
                let _ = transaction.rollback().await;
                Err(e)
            }
        }
    }
}
