use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};
use validator::Validate;

#[derive(Debug, Deserialize, ToSchema, IntoParams, Validate)]
#[into_params(parameter_in = Query)]
pub struct GetUploadUrlRequest {
    #[validate(length(min = 1))]
    pub file_name: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct GetUploadUrlResponse {
    pub upload_url: String,
    pub public_url: String,
    pub file_key: String,
}
