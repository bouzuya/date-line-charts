type MyInterceptor =
    Box<dyn FnMut(tonic::Request<()>) -> Result<tonic::Request<()>, tonic::Status>>;
type Client = google_api_proto::google::firestore::v1::firestore_client::FirestoreClient<
    tonic::service::interceptor::InterceptedService<tonic::transport::Channel, MyInterceptor>,
>;

#[derive(Clone)]
pub(crate) struct FirestoreClient {
    channel: tonic::transport::Channel,
    credential: google_cloud_auth::Credential,
}

impl FirestoreClient {
    pub(crate) async fn new<I>(scopes: I) -> Result<Self, Box<dyn std::error::Error + Send + Sync>>
    where
        I: IntoIterator,
        I::Item: Into<String>,
    {
        let channel = tonic::transport::Channel::from_static("https://firestore.googleapis.com")
            .tls_config(
                tonic::transport::ClientTlsConfig::new().domain_name("firestore.googleapis.com"),
            )?
            .connect()
            .await?;
        let credential_config = google_cloud_auth::CredentialConfig::builder()
            .scopes(scopes.into_iter().map(Into::into).collect::<Vec<String>>())
            .build()?;
        let credential = google_cloud_auth::Credential::find_default(credential_config).await?;
        Ok(Self {
            channel,
            credential,
        })
    }

    async fn client(&self) -> Result<Client, Box<dyn std::error::Error + Send + Sync>> {
        let inner = self.channel.clone();
        let token = self.credential.access_token().await?.value;
        let mut metadata_value =
            tonic::metadata::AsciiMetadataValue::try_from(format!("Bearer {}", token))?;
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
