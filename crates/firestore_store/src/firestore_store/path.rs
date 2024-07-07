use std::str::FromStr as _;

use firestore_client::{
    path::{CollectionId, DocumentId},
    CollectionPath, DocumentPath,
};
use write_model::value_object::{ChartId, EventId};

pub(crate) fn chart_collection() -> CollectionPath {
    CollectionPath::new(None, chart_collection_id())
}

pub(crate) fn chart_collection_id() -> CollectionId {
    CollectionId::from_str("charts").expect("chart collection id to be valid collection id")
}

pub(crate) fn chart_document(chart_id: ChartId) -> DocumentPath {
    chart_collection()
        .doc(DocumentId::from_str(&chart_id.to_string()).expect("chart id to be valid document id"))
        .expect("chart document path to be valid document path")
}

// events
// - event_id (pk)
// - event_stream_id + version (uk)
//
// event_streams
// - event_stream_id (pk)

pub(crate) fn event_collection_id() -> CollectionId {
    CollectionId::from_str("events").expect("event collection id to be valid collection id")
}

pub(crate) fn event_collection() -> CollectionPath {
    CollectionPath::new(None, event_collection_id())
}

#[allow(dead_code)]
pub(crate) fn event_document(event_id: EventId) -> DocumentPath {
    event_collection()
        .doc(DocumentId::from_str(&event_id.to_string()).expect("event id to be valid document id"))
        .expect("event document path to be valid document path")
}

pub(crate) fn event_stream_collection_id() -> CollectionId {
    CollectionId::from_str("event_streams")
        .expect("event_stream collection id to be valid collection id")
}

pub(crate) fn event_stream_collection() -> CollectionPath {
    CollectionPath::new(None, event_stream_collection_id())
}

pub(crate) fn event_stream_document(event_stream_id: &str) -> DocumentPath {
    event_stream_collection()
        .doc(DocumentId::from_str(event_stream_id).expect("chart id to be valid document id"))
        .expect("chart event stream document path to be valid document path")
}
