use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Debug, Deserialize, Serialize, Validate)]
pub struct CreateProductCategoryRequest {
    #[validate(required(message = "Name is required"))]
    #[validate(length(min = 1, message = "Name is required"))]
    pub name: Option<String>,
}
