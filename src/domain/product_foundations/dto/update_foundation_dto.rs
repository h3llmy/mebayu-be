use serde::Deserialize;
use utoipa::ToSchema;
use validator::Validate;

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct UpdateProductFoundationRequest {
    #[validate(length(min = 1, max = 255))]
    pub name: Option<String>,
}
