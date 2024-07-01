#[ignore = "it requires a Firestore instance to be running"]
#[tokio::test]
async fn test() -> anyhow::Result<()> {
    use anyhow::Context as _;
    use firestore_client::{DocumentPath, FirestoreClient};
    use std::str::FromStr as _;

    #[derive(Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
    struct DocumentData {
        n: i64,
    }

    let client = FirestoreClient::new()
        .await
        .map_err(|e| anyhow::anyhow!(e))?;
    assert!(client
        .get_document::<DocumentData>(&DocumentPath::from_str("col/doc")?)
        .await?
        .is_none());

    client
        .create_document(
            &DocumentPath::from_str("col/doc")?,
            &DocumentData { n: 123 },
        )
        .await?;
    assert_eq!(
        client
            .get_document::<DocumentData>(&DocumentPath::from_str("col/doc")?)
            .await?
            .context("document is some")?
            .fields,
        DocumentData { n: 123 }
    );

    client
        .update_document(
            &DocumentPath::from_str("col/doc")?,
            &DocumentData { n: 456 },
        )
        .await?;
    assert_eq!(
        client
            .get_document::<DocumentData>(&DocumentPath::from_str("col/doc")?)
            .await?
            .context("document is some")?
            .fields,
        DocumentData { n: 456 }
    );

    client
        .delete_document(&DocumentPath::from_str("col/doc")?)
        .await?;
    assert!(client
        .get_document::<DocumentData>(&DocumentPath::from_str("col/doc")?)
        .await?
        .is_none());

    Ok(())
}
