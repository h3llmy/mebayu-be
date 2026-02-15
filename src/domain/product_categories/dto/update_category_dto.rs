use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Debug, Deserialize, Serialize, Validate)]
pub struct UpdateProductCategoryRequest {
    #[validate(length(min = 1, message = "Name must not be empty"))]
    pub name: Option<String>,
}
