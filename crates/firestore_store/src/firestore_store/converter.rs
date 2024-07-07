use std::str::FromStr as _;

use firestore_client::Document;
use write_model::{
    aggregate::chart::{
        event::{Created, Deleted, Updated},
        Event, EventData,
    },
    value_object::{ChartId, DateTime},
};

use crate::schema::{self, ChartDocumentData, ChartEventDataDocumentData, EventDocumentData};

pub(crate) fn query_data_from_document(
    document: Document<ChartDocumentData>,
) -> Result<query_use_case::port::ChartQueryData, Box<dyn std::error::Error + Send + Sync>> {
    Ok(query_use_case::port::ChartQueryData {
        created_at: DateTime::from_str(&document.fields.created_at)?,
        id: ChartId::from_str(document.name.document_id().as_ref())?,
        title: document.fields.title,
    })
}

pub(crate) fn chart_event_from_document(
    document: Document<EventDocumentData<ChartEventDataDocumentData>>,
) -> Result<Event, Box<dyn std::error::Error + Send + Sync>> {
    Ok(Event {
        at: DateTime::from_str(&document.fields.at)?,
        data: match document.fields.data {
            ChartEventDataDocumentData::Created(data) => {
                EventData::Created(Created { title: data.title })
            }
            ChartEventDataDocumentData::Deleted(_) => EventData::Deleted(Deleted {}),
            ChartEventDataDocumentData::Updated(data) => {
                EventData::Updated(Updated { title: data.title })
            }
        },
        id: write_model::value_object::EventId::from_str(document.name.document_id().as_ref())?,
        stream_id: write_model::value_object::ChartId::from_str(&document.fields.stream_id)?,
        version: write_model::value_object::Version::try_from(document.fields.version)?,
    })
}

pub(crate) fn document_data_from_chart_event_data(
    event_data: &write_model::aggregate::chart::EventData,
) -> ChartEventDataDocumentData {
    match event_data {
        write_model::aggregate::chart::EventData::Created(data) => {
            ChartEventDataDocumentData::Created(schema::Created {
                title: data.title.to_owned(),
            })
        }
        write_model::aggregate::chart::EventData::Deleted(_) => {
            ChartEventDataDocumentData::Deleted(schema::Deleted {})
        }
        write_model::aggregate::chart::EventData::Updated(data) => {
            ChartEventDataDocumentData::Updated(schema::Updated {
                title: data.title.to_owned(),
            })
        }
    }
}
