use std::str::FromStr as _;

use firestore_client::Document;
use write_model::{
    event::{
        ChartCreated, ChartDeleted, ChartEvent, ChartEventData, ChartUpdated, DataPointCreated,
        DataPointDeleted, DataPointEvent, DataPointEventData, DataPointUpdated, Event,
    },
    value_object::{ChartId, DateTime, XValue, YValue},
};

use crate::schema::{
    self, ChartDocumentData, ChartEventDataDocumentData, DataPointDocumentData,
    DataPointEventDataDocumentData, EventDataDocumentData, EventDocumentData,
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
        data: match document.fields.data {
            EventDataDocumentData::Chart(event_data) => match event_data {
                ChartEventDataDocumentData::Created(data) => {
                    ChartEventData::Created(ChartCreated { title: data.title })
                }
                ChartEventDataDocumentData::Deleted(_) => ChartEventData::Deleted(ChartDeleted {}),
                ChartEventDataDocumentData::Updated(data) => {
                    ChartEventData::Updated(ChartUpdated { title: data.title })
                }
            },
            EventDataDocumentData::DataPoint(_) => unreachable!(),
        },
        id: write_model::value_object::EventId::from_str(document.name.document_id().as_ref())?,
        stream_id: write_model::value_object::ChartId::from_str(&document.fields.stream_id)?,
        version: write_model::value_object::Version::try_from(document.fields.version)?,
    })
}

pub(crate) fn data_point_event_from_document(
    document: Document<EventDocumentData>,
) -> Result<DataPointEvent, Box<dyn std::error::Error + Send + Sync>> {
    Ok(DataPointEvent {
        at: DateTime::from_str(&document.fields.at)?,
        data: match document.fields.data {
            EventDataDocumentData::Chart(_) => unreachable!(),
            EventDataDocumentData::DataPoint(event_data) => match event_data {
                DataPointEventDataDocumentData::Created(data) => {
                    DataPointEventData::Created(DataPointCreated {
                        value: YValue::from(u32::try_from(data.value)?),
                    })
                }
                DataPointEventDataDocumentData::Deleted(_) => {
                    DataPointEventData::Deleted(DataPointDeleted {})
                }
                DataPointEventDataDocumentData::Updated(data) => {
                    DataPointEventData::Updated(DataPointUpdated {
                        value: YValue::from(u32::try_from(data.value)?),
                    })
                }
            },
        },
        id: write_model::value_object::EventId::from_str(document.name.document_id().as_ref())?,
        stream_id: write_model::value_object::DataPointId::from_str(&document.fields.stream_id)?,
        version: write_model::value_object::Version::try_from(document.fields.version)?,
    })
}

pub(crate) fn event_from_document(
    document: Document<EventDocumentData>,
) -> Result<Event, Box<dyn std::error::Error + Send + Sync>> {
    match document.fields.data {
        EventDataDocumentData::Chart(_) => chart_event_from_document(document).map(Event::from),
        EventDataDocumentData::DataPoint(_) => {
            data_point_event_from_document(document).map(Event::from)
        }
    }
}

pub(crate) fn event_document_data_from_event(event: &Event) -> EventDocumentData {
    match event {
        Event::Chart(event) => event_document_data_from_chart_event(event),
        Event::DataPoint(event) => event_document_data_from_data_point_event(event),
    }
}

pub(crate) fn event_document_data_from_chart_event(event: &ChartEvent) -> EventDocumentData {
    EventDocumentData {
        at: event.at.to_string(),
        data: EventDataDocumentData::Chart(document_data_from_chart_event_data(&event.data)),
        id: event.id.to_string(),
        stream_id: event.stream_id.to_string(),
        version: i64::from(event.version),
    }
}

pub(crate) fn event_document_data_from_data_point_event(
    event: &DataPointEvent,
) -> EventDocumentData {
    EventDocumentData {
        at: event.at.to_string(),
        data: EventDataDocumentData::DataPoint(document_data_from_data_point_event_data(
            &event.data,
        )),
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
            ChartEventDataDocumentData::Created(schema::chart_event_data_document_data::Created {
                title: data.title.to_owned(),
            })
        }
        write_model::event::ChartEventData::Deleted(_) => {
            ChartEventDataDocumentData::Deleted(schema::chart_event_data_document_data::Deleted {})
        }
        write_model::event::ChartEventData::Updated(data) => {
            ChartEventDataDocumentData::Updated(schema::chart_event_data_document_data::Updated {
                title: data.title.to_owned(),
            })
        }
    }
}

fn document_data_from_data_point_event_data(
    event_data: &write_model::event::DataPointEventData,
) -> DataPointEventDataDocumentData {
    match event_data {
        write_model::event::DataPointEventData::Created(data) => {
            DataPointEventDataDocumentData::Created(
                schema::data_point_event_data_document_data::Created {
                    value: i64::from(u32::from(data.value)),
                },
            )
        }
        write_model::event::DataPointEventData::Deleted(_) => {
            DataPointEventDataDocumentData::Deleted(
                schema::data_point_event_data_document_data::Deleted {},
            )
        }
        write_model::event::DataPointEventData::Updated(data) => {
            DataPointEventDataDocumentData::Updated(
                schema::data_point_event_data_document_data::Updated {
                    value: i64::from(u32::from(data.value)),
                },
            )
        }
    }
}
