use std::str::FromStr as _;

use firestore_client::Document;
use write_model::{
    event::{ChartCreated, ChartDeleted, ChartEvent, ChartEventData, ChartUpdated},
    value_object::{ChartId, DateTime, XValue, YValue},
};

use crate::schema::{
    self, ChartDocumentData, ChartEventDataDocumentData, DataPointDocumentData, EventDocumentData,
};

pub(crate) fn query_data_from_document(
    document: Document<ChartDocumentData>,
) -> Result<query_use_case::port::ChartQueryData, Box<dyn std::error::Error + Send + Sync>> {
    Ok(query_use_case::port::ChartQueryData {
        created_at: DateTime::from_str(&document.fields.created_at)?,
        id: ChartId::from_str(document.name.document_id().as_ref())?,
        title: document.fields.title,
    })
}

pub(crate) fn data_point_query_data_from_document(
    document: Document<DataPointDocumentData>,
) -> Result<query_use_case::port::DataPointQueryData, Box<dyn std::error::Error + Send + Sync>> {
    Ok(query_use_case::port::DataPointQueryData {
        chart_id: ChartId::from_str(&document.fields.chart_id)?,
        created_at: DateTime::from_str(&document.fields.created_at)?,
        x_value: XValue::from_str(&document.fields.x_value)?,
        y_value: YValue::from(u32::try_from(document.fields.y_value)?),
    })
}

pub(crate) fn chart_event_from_document(
    document: Document<EventDocumentData>,
) -> Result<ChartEvent, Box<dyn std::error::Error + Send + Sync>> {
    Ok(ChartEvent {
        at: DateTime::from_str(&document.fields.at)?,
        data: match serde_json::from_str::<ChartEventDataDocumentData>(&document.fields.data)? {
            ChartEventDataDocumentData::Created(data) => {
                ChartEventData::Created(ChartCreated { title: data.title })
            }
            ChartEventDataDocumentData::Deleted(_) => ChartEventData::Deleted(ChartDeleted {}),
            ChartEventDataDocumentData::Updated(data) => {
                ChartEventData::Updated(ChartUpdated { title: data.title })
            }
        },
        id: write_model::value_object::EventId::from_str(document.name.document_id().as_ref())?,
        stream_id: write_model::value_object::ChartId::from_str(&document.fields.stream_id)?,
        version: write_model::value_object::Version::try_from(document.fields.version)?,
    })
}

pub(crate) fn event_document_data_from_event(event: &ChartEvent) -> EventDocumentData {
    EventDocumentData {
        at: event.at.to_string(),
        data: serde_json::to_string(&document_data_from_chart_event_data(&event.data)).unwrap(),
        id: event.id.to_string(),
        stream_id: event.stream_id.to_string(),
        version: i64::from(event.version),
    }
}

fn document_data_from_chart_event_data(
    event_data: &write_model::event::ChartEventData,
) -> ChartEventDataDocumentData {
    match event_data {
        write_model::event::ChartEventData::Created(data) => {
            ChartEventDataDocumentData::Created(schema::Created {
                title: data.title.to_owned(),
            })
        }
        write_model::event::ChartEventData::Deleted(_) => {
            ChartEventDataDocumentData::Deleted(schema::Deleted {})
        }
        write_model::event::ChartEventData::Updated(data) => {
            ChartEventDataDocumentData::Updated(schema::Updated {
                title: data.title.to_owned(),
            })
        }
    }
}
