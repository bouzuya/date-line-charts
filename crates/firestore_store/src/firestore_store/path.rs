mod event_stream;

use std::str::FromStr as _;

use firestore_client::{
    path::{CollectionId, DocumentId},
    CollectionPath, DocumentPath,
};
use write_model::value_object::{ChartId, DataPointId, EventId};

pub(crate) use self::event_stream::event_stream_document;

pub(crate) fn query_updater_document() -> DocumentPath {
    CollectionPath::new(
        None,
        CollectionId::from_str("query").expect("query collection id to be valid"),
    )
    .doc(DocumentId::from_str("updater").expect("updater document id to be valid"))
    .expect("query updater document path to be valid")
}

pub(crate) fn query_updater_processed_event_document(event_id: EventId) -> DocumentPath {
    query_updater_document()
        .collection("processed_events")
        .expect("query updater processed event collection path to be valid")
        .doc(DocumentId::from_str(&event_id.to_string()).expect("event id to be valid"))
        .expect("query updater processed event document path to be valid")
}

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

pub(crate) fn data_point_collection(chart_id: ChartId) -> CollectionPath {
    chart_document(chart_id)
        .collection(
            CollectionId::from_str("data_points").expect("data point collection id to be valid"),
        )
        .expect("data point collection path to be valid")
}

pub(crate) fn data_point_document(data_point_id: DataPointId) -> DocumentPath {
    data_point_collection(data_point_id.chart_id())
        .doc(DocumentId::from_str(&data_point_id.to_string()).expect("data point id to be valid"))
        .expect("data point document path to be valid")
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
