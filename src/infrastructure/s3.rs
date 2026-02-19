use crate::core::config::Config;
use aws_config::{BehaviorVersion, Region};
use aws_sdk_s3::{Client, config::Credentials};

pub async fn create_s3_client(config: &Config) -> Client {
    let credentials = Credentials::new(
        config.s3_access_key.clone(),
        config.s3_secret_key.clone(),
        None,
        None,
        "custom",
    );

    let shared_config = aws_config::defaults(BehaviorVersion::latest())
        .region(Region::new(config.s3_region.clone()))
        .endpoint_url(config.s3_endpoint.clone())
        .credentials_provider(credentials)
        .load()
        .await;

    Client::new(&shared_config)
}

pub async fn get_presigned_url(
    client: &Client,
    bucket: &str,
    key: &str,
    expires_in: std::time::Duration,
) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    let presigned_request = client
        .put_object()
        .bucket(bucket)
        .key(key)
        .presigned(aws_sdk_s3::presigning::PresigningConfig::expires_in(
            expires_in,
        )?)
        .await?;

    Ok(presigned_request.uri().to_string())
}
