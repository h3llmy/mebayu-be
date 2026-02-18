use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use validator::Validate;

#[derive(Debug, Deserialize, Serialize, Validate, ToSchema)]
pub struct UpdateProductCategoryRequest {
    #[validate(length(min = 1))]
    pub name: Option<String>,
}
