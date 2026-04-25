use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use validator::Validate;

#[derive(Debug, Serialize, Deserialize, Validate, ToSchema)]
pub struct CreateLanguageRequest {
    #[validate(length(min = 2, max = 10))]
    pub code: String,
    #[validate(length(min = 1, max = 50))]
    pub name: String,
    pub is_default: bool,
}

#[derive(Debug, Serialize, Deserialize, Validate, ToSchema)]
pub struct UpdateLanguageRequest {
    #[validate(length(min = 2, max = 10))]
    pub code: Option<String>,
    #[validate(length(min = 1, max = 50))]
    pub name: Option<String>,
    pub is_default: Option<bool>,
}
