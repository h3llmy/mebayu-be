use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use validator::Validate;

#[derive(Debug, Serialize, Deserialize, ToSchema, Validate)]
pub struct GetUploadUrlRequest {
    #[validate(length(min = 1))]
    pub path: String,

    #[validate(length(min = 1))]
    pub metadata: Vec<FileUploadMetadata>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema, Validate)]
pub struct FileUploadMetadata {
    #[validate(length(min = 1))]
    pub content_type: String,

    #[validate(length(min = 1))]
    pub file_name: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct GetUploadUrlResponse {
    pub upload_url: String,
    pub public_url: String,
    pub file_key: String,
}
