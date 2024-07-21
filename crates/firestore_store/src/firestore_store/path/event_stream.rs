use std::str::FromStr as _;

use firestore_client::{
    path::{CollectionId, DocumentId},
    CollectionPath, DocumentPath,
};

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
