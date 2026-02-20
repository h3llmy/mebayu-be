use aws_config::{BehaviorVersion, Region};
use aws_sdk_s3::{
    Client,
    config::{Builder as S3ConfigBuilder, Credentials, SharedCredentialsProvider},
};
use std::time::Duration;
use uuid::Uuid;

use crate::{core::config::Config, shared::dto::object_storage::GetUploadUrlRequest};
use crate::{core::error::AppError, shared::dto::object_storage::GetUploadUrlResponse};

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
        req: GetUploadUrlRequest,
        expires_in: Duration,
    ) -> Result<Vec<GetUploadUrlResponse>, AppError> {
        let mut results = Vec::new();

        for file in req.metadata {
            // extract extension
            let extension = std::path::Path::new(&file.file_name)
                .extension()
                .and_then(|ext| ext.to_str())
                .unwrap_or("png");

            let file_key = format!("{}/{}.{}", &req.path, Uuid::new_v4(), extension);

            let presigned = self
                .client
                .put_object()
                .bucket(&self.bucket)
                .key(&file_key)
                .content_type(&file.content_type)
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

            results.push(GetUploadUrlResponse {
                file_key,
                public_url,
                upload_url,
            });
        }

        Ok(results)
    }

    pub async fn validate_object(&self, url: &str) -> Result<(), AppError> {
        let key = self.extract_key(url).map_err(|e| {
            AppError::Validation(
                vec![("image_urls".to_string(), vec![e])]
                    .into_iter()
                    .collect(),
            )
        })?;

        self.client
            .head_object()
            .bucket(&self.bucket)
            .key(&key)
            .send()
            .await
            .map_err(|_| {
                AppError::Validation(
                    vec![(
                        "image_urls".to_string(),
                        vec![format!("File not found in storage: {}", url)],
                    )]
                    .into_iter()
                    .collect(),
                )
            })?;

        Ok(())
    }

    fn extract_key(&self, url: &str) -> Result<String, String> {
        // Expected format: http://.../bucket/key
        // For MinIO/S3 with path style: http://host:port/bucket/key

        let bucket_part = format!("/{}/", self.bucket);
        if let Some(pos) = url.find(&bucket_part) {
            Ok(url[pos + bucket_part.len()..].to_string())
        } else {
            Err("Invalid storage URL format".to_string())
        }
    }
}
