#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub(crate) struct ChartDocumentData {
    pub(crate) created_at: String,
    pub(crate) title: String,
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub(crate) struct EventStreamDocumentData {
    pub(crate) id: String,
    pub(crate) last_event_at: String,
    pub(crate) version: i64,
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub(crate) struct EventDocumentData<T> {
    pub(crate) at: String,
    pub(crate) data: T,
    pub(crate) id: String,
    pub(crate) stream_id: String,
    pub(crate) version: i64,
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "snake_case", tag = "type")]
pub(crate) enum ChartEventDataDocumentData {
    Created(Created),
    Deleted(Deleted),
    Updated(Updated),
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct Created {
    pub(crate) title: String,
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct Deleted {}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct Updated {
    pub(crate) title: String,
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
                data: ChartEventDataDocumentData::Created(Created {
                    title: "title".to_owned(),
                }),
                id: "id".to_owned(),
                stream_id: "stream_id".to_owned(),
                version: 1,
            })?,
            serde_json::json!({
                "at": "2020-01-02T03:04:05.678Z",
                "data": {
                    "title": "title",
                    "type": "created",
                },
                "id": "id",
                "stream_id": "stream_id",
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
                data: ChartEventDataDocumentData::Deleted(Deleted {}),
                id: "id".to_owned(),
                stream_id: "stream_id".to_owned(),
                version: 1,
            })?,
            serde_json::json!({
                "at": "2020-01-02T03:04:05.678Z",
                "data": {
                    "type": "deleted",
                },
                "id": "id",
                "stream_id": "stream_id",
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
                data: ChartEventDataDocumentData::Updated(Updated {
                    title: "title".to_owned(),
                }),
                id: "id".to_owned(),
                stream_id: "stream_id".to_owned(),
                version: 1,
            })?,
            serde_json::json!({
                "at": "2020-01-02T03:04:05.678Z",
                "data": {
                    "title": "title",
                    "type": "updated",
                },
                "id": "id",
                "stream_id": "stream_id",
                "version": 1,
            }),
        );
        Ok(())
    }
}
