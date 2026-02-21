use aws_config::{BehaviorVersion, Region};
use aws_sdk_s3::{
    Client,
    config::{Builder as S3ConfigBuilder, Credentials, SharedCredentialsProvider},
};
use std::time::Duration;
use uuid::Uuid;

use crate::{
    core::{config::Config, error::AppError},
    shared::dto::object_storage::{GetUploadUrlRequest, GetUploadUrlResponse},
};

#[cfg_attr(test, mockall::automock)]
#[async_trait::async_trait]
pub trait Storage: Send + Sync {
    async fn generate_upload_url(
        &self,
        req: GetUploadUrlRequest,
        expires_in: Duration,
    ) -> Result<Vec<GetUploadUrlResponse>, AppError>;

    async fn validate_object(&self, url: &str) -> Result<(), AppError>;
}

pub struct S3Service {
    client: Client,
    bucket: String,
    public_url: String,
}

#[async_trait::async_trait]
impl Storage for S3Service {
    async fn generate_upload_url(
        &self,
        req: GetUploadUrlRequest,
        expires_in: Duration,
    ) -> Result<Vec<GetUploadUrlResponse>, AppError> {
        let mut results = Vec::new();

        for file in req.metadata {
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

    async fn validate_object(&self, url: &str) -> Result<(), AppError> {
        let key = Self::extract_key(&self.bucket, url).map_err(|e| {
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

    /// Pure helper — fully unit testable
    fn extract_key(bucket: &str, url: &str) -> Result<String, String> {
        let bucket_part = format!("/{}/", bucket);

        if let Some(pos) = url.find(&bucket_part) {
            Ok(url[pos + bucket_part.len()..].to_string())
        } else {
            Err("Invalid storage URL format".to_string())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shared::dto::object_storage::FileUploadMetadata;
    use mockall::predicate::*;
    use std::time::Duration;

    // -------------------------
    // extract_key tests
    // -------------------------

    #[test]
    fn test_extract_key_success() {
        let bucket = "my-bucket";
        let url = "http://localhost:9000/my-bucket/uploads/test.png";

        let key = S3Service::extract_key(bucket, url).unwrap();

        assert_eq!(key, "uploads/test.png");
    }

    #[test]
    fn test_extract_key_invalid_format() {
        let bucket = "my-bucket";
        let url = "http://localhost:9000/wrong-bucket/test.png";

        let result = S3Service::extract_key(bucket, url);

        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Invalid storage URL format");
    }

    // -------------------------
    // Storage trait mocking tests
    // -------------------------

    #[tokio::test]
    async fn test_validate_object_success_mocked() {
        let mut mock = MockStorage::new();

        mock.expect_validate_object()
            .with(eq("http://localhost:9000/my-bucket/uploads/test.png"))
            .times(1)
            .returning(|_| Ok(()));

        let result = mock
            .validate_object("http://localhost:9000/my-bucket/uploads/test.png")
            .await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_validate_object_failure_mocked() {
        let mut mock = MockStorage::new();

        mock.expect_validate_object().returning(|url| {
            Err(AppError::Validation(
                vec![(
                    "image_urls".to_string(),
                    vec![format!("File not found in storage: {}", url)],
                )]
                .into_iter()
                .collect(),
            ))
        });

        let result = mock
            .validate_object("http://localhost:9000/my-bucket/uploads/test.png")
            .await;

        assert!(matches!(result, Err(AppError::Validation(_))));
    }

    #[tokio::test]
    async fn test_generate_upload_url_mocked() {
        let mut mock = MockStorage::new();

        let request = GetUploadUrlRequest {
            path: "uploads".to_string(),
            metadata: vec![FileUploadMetadata {
                file_name: "image.png".to_string(),
                content_type: "image/png".to_string(),
            }],
        };

        mock.expect_generate_upload_url()
            .times(1)
            .returning(|req, _| {
                Ok(vec![GetUploadUrlResponse {
                    file_key: format!("{}/test.png", req.path),
                    public_url: "http://localhost:9000/my-bucket/uploads/test.png".to_string(),
                    upload_url: "http://presigned-url".to_string(),
                }])
            });

        let result = mock
            .generate_upload_url(request, Duration::from_secs(300))
            .await
            .unwrap();

        assert_eq!(result.len(), 1);
        assert!(result[0].upload_url.contains("http"));
    }
}
