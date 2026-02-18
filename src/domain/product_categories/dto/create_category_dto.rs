use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Debug, Deserialize, Serialize, Validate)]
pub struct CreateProductCategoryRequest {
    #[validate(required)]
    #[validate(length(min = 1))]
    pub name: Option<String>,
}
