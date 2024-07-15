pub(crate) mod chart_event_data_document_data;
pub(crate) mod data_point_event_data_document_data;

pub(crate) use chart_event_data_document_data::ChartEventDataDocumentData;
pub(crate) use data_point_event_data_document_data::DataPointEventDataDocumentData;

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub(crate) struct ChartDocumentData {
    pub(crate) created_at: String,
    pub(crate) title: String,
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub(crate) struct DataPointDocumentData {
    pub(crate) chart_id: String,
    pub(crate) created_at: String,
    pub(crate) x_value: String,
    pub(crate) y_value: i64,
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub(crate) struct EventStreamDocumentData {
    pub(crate) id: String,
    pub(crate) last_event_at: String,
    pub(crate) version: i64,
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub(crate) struct EventDocumentData {
    pub(crate) at: String,
    #[serde(flatten)]
    pub(crate) data: EventDataDocumentData,
    pub(crate) id: String,
    pub(crate) stream_id: String,
    pub(crate) version: i64,
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "snake_case", tag = "stream_type")]
pub(crate) enum EventDataDocumentData {
    Chart(ChartEventDataDocumentData),
    DataPoint(DataPointEventDataDocumentData),
}

#[derive(serde::Deserialize, serde::Serialize)]
pub(crate) struct UpdaterMetadataDocumentData {
    pub(crate) last_processed_event_at: String,
}

#[derive(serde::Deserialize, serde::Serialize)]
pub(crate) struct UpdaterMetadataProcessedEventDocumentData {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_created() -> anyhow::Result<()> {
        assert_eq!(
            serde_json::to_value(EventDocumentData {
                at: "2020-01-02T03:04:05.678Z".to_owned(),
                data: EventDataDocumentData::Chart(ChartEventDataDocumentData::Created(
                    chart_event_data_document_data::Created {
                        title: "title".to_owned(),
                    }
                )),
                id: "id".to_owned(),
                stream_id: "stream_id".to_owned(),
                version: 1,
            })?,
            serde_json::json!({
                "at": "2020-01-02T03:04:05.678Z",
                "data": {
                    "title":"title"
                },
                "id": "id",
                "stream_id": "stream_id",
                "stream_type": "chart",
                "type": "created",
                "version": 1,
            }),
        );
        Ok(())
    }

    #[test]
    fn test_deleted() -> anyhow::Result<()> {
        assert_eq!(
            serde_json::to_value(EventDocumentData {
                at: "2020-01-02T03:04:05.678Z".to_owned(),
                data: EventDataDocumentData::Chart(ChartEventDataDocumentData::Deleted(
                    chart_event_data_document_data::Deleted {}
                )),
                id: "id".to_owned(),
                stream_id: "stream_id".to_owned(),
                version: 1,
            })?,
            serde_json::json!({
                "at": "2020-01-02T03:04:05.678Z",
                "data": {},
                "id": "id",
                "stream_id": "stream_id",
                "stream_type": "chart",
                "type": "deleted",
                "version": 1,
            }),
        );
        Ok(())
    }

    #[test]
    fn test_updated() -> anyhow::Result<()> {
        assert_eq!(
            serde_json::to_value(EventDocumentData {
                at: "2020-01-02T03:04:05.678Z".to_owned(),
                data: EventDataDocumentData::Chart(ChartEventDataDocumentData::Updated(
                    chart_event_data_document_data::Updated {
                        title: "title".to_owned(),
                    }
                )),
                id: "id".to_owned(),
                stream_id: "stream_id".to_owned(),
                version: 1,
            })?,
            serde_json::json!({
                "at": "2020-01-02T03:04:05.678Z",
                "data": {
                    "title": "title"
                },
                "id": "id",
                "stream_id": "stream_id",
                "stream_type": "chart",
                "type": "updated",
                "version": 1,
            }),
        );
        Ok(())
    }
}
