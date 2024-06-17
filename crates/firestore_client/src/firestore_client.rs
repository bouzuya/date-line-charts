use std::collections::BTreeMap;
use std::str::FromStr;
use std::sync::Arc;

pub use firestore_path as path;
use firestore_path::DatabaseName;

pub use firestore_path::CollectionPath;
pub use firestore_path::DocumentName;
pub use firestore_path::DocumentPath;
pub use serde_firestore_value::Timestamp;

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct Document<T> {
    pub name: DocumentName,
    pub fields: T,
    pub create_time: Timestamp,
    pub update_time: Timestamp,
}

fn document_from_google_api_proto_document<T>(
    google_api_proto::google::firestore::v1::Document {
        name,
        fields,
        create_time,
        update_time,
    }: google_api_proto::google::firestore::v1::Document,
) -> Result<Document<T>, Error>
where
    T: serde::de::DeserializeOwned,
{
    Ok(Document::<T> {
        name: DocumentName::from_str(&name).expect("document.name to be valid document_name"),
        fields: serde_firestore_value::from_value::<T>(
            &google_api_proto::google::firestore::v1::Value {
                value_type: Some(
                    google_api_proto::google::firestore::v1::value::ValueType::MapValue(
                        google_api_proto::google::firestore::v1::MapValue { fields },
                    ),
                ),
            },
        )
        .map_err(InnerError::Deserialize)?,
        create_time: serde_firestore_value::Timestamp::from(
            create_time.expect("document.create_time to be set"),
        ),
        update_time: serde_firestore_value::Timestamp::from(
            update_time.expect("document.update_time to be set"),
        ),
    })
}

#[derive(Debug, thiserror::Error)]
#[error(transparent)]
pub struct Error(#[from] InnerError);

#[derive(Debug, thiserror::Error)]
enum InnerError {
    #[error("deserialize")]
    Deserialize(#[source] serde_firestore_value::Error),
    #[error("header value")]
    HeaderValue(#[source] tonic::metadata::errors::InvalidMetadataValue),
    #[error("no map value")]
    NoMapValue,
    #[error("project_id")]
    ProjectId(#[source] firestore_path::Error),
    #[error("serialize")]
    Serialize(#[source] serde_firestore_value::Error),
    #[error("status")]
    Status(#[source] tonic::Status),
    #[error("token")]
    Token(#[source] Box<dyn std::error::Error + Send + Sync>),
}

type MyInterceptor =
    Box<dyn FnMut(tonic::Request<()>) -> Result<tonic::Request<()>, tonic::Status> + Send + Sync>;
type Client = google_api_proto::google::firestore::v1::firestore_client::FirestoreClient<
    tonic::service::interceptor::InterceptedService<tonic::transport::Channel, MyInterceptor>,
>;

#[derive(Clone)]
pub struct FirestoreClient {
    channel: tonic::transport::Channel,
    database_name: firestore_path::DatabaseName,
    token_source: Arc<dyn google_cloud_token::TokenSource>,
}

impl FirestoreClient {
    pub async fn new() -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let default_token_source_provider =
            google_cloud_auth::token::DefaultTokenSourceProvider::new(
                google_cloud_auth::project::Config {
                    scopes: Some(&[
                        "https://www.googleapis.com/auth/cloud-platform",
                        "https://www.googleapis.com/auth/datastore",
                    ]),
                    ..Default::default()
                },
            )
            .await?;
        let token_source =
            google_cloud_token::TokenSourceProvider::token_source(&default_token_source_provider);
        let project_id = default_token_source_provider
            .project_id
            .ok_or("project_id not found")?;
        let channel = tonic::transport::Channel::from_static("https://firestore.googleapis.com")
            .tls_config(
                tonic::transport::ClientTlsConfig::new().domain_name("firestore.googleapis.com"),
            )?
            .connect()
            .await?;
        let database_name =
            DatabaseName::from_project_id(project_id).map_err(InnerError::ProjectId)?;
        Ok(Self {
            channel,
            database_name,
            token_source,
        })
    }

    pub async fn create_document<T>(
        &self,
        document_path: &DocumentPath,
        document_data: &T,
    ) -> Result<(), Error>
    where
        T: serde::Serialize,
    {
        let mut client = self.client().await?;
        client
            .create_document(
                google_api_proto::google::firestore::v1::CreateDocumentRequest {
                    parent: document_path
                        .parent()
                        .parent()
                        .map(|document_path| {
                            self.database_name
                                .doc(document_path.clone())
                                .expect("document_path to be valid document_name")
                                .to_string()
                        })
                        .unwrap_or_else(|| self.database_name.root_document_name().to_string()),
                    collection_id: document_path.collection_id().to_string(),
                    document_id: document_path.document_id().to_string(),
                    document: Some(google_api_proto::google::firestore::v1::Document {
                        name: String::default(),
                        fields: fields_from_document_data(&document_data)?,
                        create_time: None,
                        update_time: None,
                    }),
                    mask: None,
                },
            )
            .await
            .map(|response| response.into_inner())
            .map_err(InnerError::Status)?;
        Ok(())
    }

    pub async fn get_document<T>(
        &self,
        document_path: &DocumentPath,
    ) -> Result<Option<Document<T>>, Error>
    where
        T: serde::de::DeserializeOwned,
    {
        let mut client = self.client().await?;
        client
            .get_document(
                google_api_proto::google::firestore::v1::GetDocumentRequest {
                    name: self
                        .database_name
                        .doc(document_path.clone())
                        .expect("document_path to be valid document_name")
                        .to_string(),
                    mask: None,
                    consistency_selector: None,
                },
            )
            .await
            .map(|response| Some(response.into_inner()))
            .or_else(|status| match status.code() {
                tonic::Code::NotFound => Ok(None),
                _ => Err(InnerError::Status(status)),
            })?
            .map(document_from_google_api_proto_document::<T>)
            .transpose()
    }

    pub async fn list_all_documents<T>(
        &self,
        collection_path: &CollectionPath,
    ) -> Result<Vec<Document<T>>, Error>
    where
        T: serde::de::DeserializeOwned,
    {
        let mut client = self.client().await?;
        let mut page_token = String::default();
        let mut all_documents = Vec::default();
        loop {
            let google_api_proto::google::firestore::v1::ListDocumentsResponse {
                documents,
                next_page_token,
            } = client
                .list_documents(
                    google_api_proto::google::firestore::v1::ListDocumentsRequest {
                        parent: collection_path
                            .parent()
                            .map(|document_path| {
                                self.database_name
                                    .doc(document_path.clone())
                                    .expect("document_path to be valid document_name")
                                    .to_string()
                            })
                            .unwrap_or_else(|| self.database_name.root_document_name().to_string()),
                        collection_id: collection_path.collection_id().to_string(),
                        page_size: 65535,
                        page_token: page_token.clone(),
                        order_by: String::default(),
                        mask: None,
                        show_missing: false,
                        consistency_selector: None,
                    },
                )
                .await
                .map_err(InnerError::Status)?
                .into_inner();
            let new_documents = documents
                .into_iter()
                .map(document_from_google_api_proto_document::<T>)
                .collect::<Result<Vec<Document<T>>, Error>>()?;
            all_documents.extend(new_documents);
            page_token = next_page_token;
            if page_token.is_empty() {
                break;
            }
        }
        Ok(all_documents)
    }

    pub async fn update_document<T>(
        &self,
        document_path: &DocumentPath,
        document_data: &T,
    ) -> Result<(), Error>
    where
        T: serde::Serialize,
    {
        let mut client = self.client().await?;
        client
            .update_document(
                google_api_proto::google::firestore::v1::UpdateDocumentRequest {
                    document: Some(google_api_proto::google::firestore::v1::Document {
                        name: self
                            .database_name
                            .doc(document_path.clone())
                            .expect("document_path to be valid document_name")
                            .to_string(),
                        fields: fields_from_document_data(document_data)?,
                        create_time: None,
                        update_time: None,
                    }),
                    update_mask: None,
                    mask: None,
                    current_document: None,
                },
            )
            .await
            .map(|response| response.into_inner())
            .map_err(InnerError::Status)?;
        Ok(())
    }

    async fn client(&self) -> Result<Client, Error> {
        let inner = self.channel.clone();
        let token = self.token_source.token().await.map_err(InnerError::Token)?;
        let mut metadata_value =
            tonic::metadata::AsciiMetadataValue::try_from(format!("Bearer {}", token))
                .map_err(InnerError::HeaderValue)?;
        metadata_value.set_sensitive(true);
        let interceptor: MyInterceptor = Box::new(
            move |mut request: tonic::Request<()>| -> Result<tonic::Request<()>, tonic::Status> {
                request
                    .metadata_mut()
                    .insert("authorization", metadata_value.clone());
                Ok(request)
            },
        );
        let client =
            google_api_proto::google::firestore::v1::firestore_client::FirestoreClient::with_interceptor(inner,interceptor);
        Ok(client)
    }
}

fn fields_from_document_data<T>(
    document_data: &T,
) -> Result<BTreeMap<String, google_api_proto::google::firestore::v1::Value>, InnerError>
where
    T: serde::Serialize,
{
    if let google_api_proto::google::firestore::v1::Value {
        value_type:
            Some(google_api_proto::google::firestore::v1::value::ValueType::MapValue(map_value)),
    } = serde_firestore_value::to_value(document_data).map_err(InnerError::Serialize)?
    {
        Ok(map_value.fields)
    } else {
        Err(InnerError::NoMapValue)
    }
}
