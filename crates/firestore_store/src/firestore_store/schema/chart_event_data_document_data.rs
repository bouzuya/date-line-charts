#[derive(Debug, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "snake_case", tag = "type")]
pub(crate) enum ChartEventDataDocumentData {
    Created(Created),
    Deleted(Deleted),
    Updated(Updated),
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub(crate) struct Created {
    pub(crate) title: String,
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub(crate) struct Deleted {}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub(crate) struct Updated {
    pub(crate) title: String,
}
