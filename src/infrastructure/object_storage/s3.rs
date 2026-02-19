use crate::core::config::Config;
use aws_config::{BehaviorVersion, Region};
use aws_sdk_s3::{
    Client,
    config::{Builder as S3ConfigBuilder, Credentials, SharedCredentialsProvider},
};

pub async fn create_s3_client(config: &Config) -> Client {
    // Static credentials for MinIO
    let credentials = Credentials::new(
        config.s3_access_key.clone(),
        config.s3_secret_key.clone(),
        None,
        None,
        "minio",
    );

    // Base AWS config
    let base_config = aws_config::defaults(BehaviorVersion::latest())
        .region(Region::new(config.s3_region.clone()))
        .credentials_provider(SharedCredentialsProvider::new(credentials))
        .endpoint_url(config.s3_endpoint.clone())
        .load()
        .await;

    // 🔥 Important for MinIO
    let s3_config = S3ConfigBuilder::from(&base_config)
        .force_path_style(true) // REQUIRED for MinIO
        .build();

    Client::from_conf(s3_config)
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
