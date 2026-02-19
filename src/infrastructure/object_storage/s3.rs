use aws_config::{BehaviorVersion, Region};
use aws_sdk_s3::{
    Client,
    config::{Builder as S3ConfigBuilder, Credentials, SharedCredentialsProvider},
};
use std::time::Duration;
use uuid::Uuid;

use crate::core::config::Config;
use crate::core::error::AppError;

pub struct S3Service {
    client: Client,
    bucket: String,
    public_url: String,
}

impl S3Service {
    pub async fn new(config: &Config) -> Self {
        let credentials = Credentials::new(
            config.s3_access_key.clone(),
            config.s3_secret_key.clone(),
            None,
            None,
            "minio",
        );

        let base_config = aws_config::defaults(BehaviorVersion::latest())
            .region(Region::new(config.s3_region.clone()))
            .credentials_provider(SharedCredentialsProvider::new(credentials))
            .endpoint_url(config.s3_endpoint.clone())
            .load()
            .await;

        let s3_config = S3ConfigBuilder::from(&base_config)
            .force_path_style(true)
            .build();

        let client = Client::from_conf(s3_config);

        Self {
            client,
            bucket: config.s3_bucket.clone(),
            public_url: config.s3_endpoint.clone(),
        }
    }

    pub async fn generate_upload_url(
        &self,
        original_file_name: &str,
        path: &str,
        expires_in: Duration,
    ) -> Result<(String, String, String), AppError> {
        // extract extension
        let extension = std::path::Path::new(original_file_name)
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("png");

        let file_key = format!("{}/{}.{}", path, Uuid::new_v4(), extension);

        let presigned = self
            .client
            .put_object()
            .bucket(&self.bucket)
            .key(&file_key)
            .presigned(
                aws_sdk_s3::presigning::PresigningConfig::expires_in(expires_in)
                    .map_err(|e| AppError::Storage(e.to_string()))?,
            )
            .await
            .map_err(|e| AppError::Storage(e.to_string()))?;

        let upload_url = presigned.uri().to_string();

        let public_url = format!(
            "{}/{}/{}",
            self.public_url.trim_end_matches('/'),
            self.bucket,
            file_key
        );

        Ok((upload_url, public_url, file_key))
    }
}
