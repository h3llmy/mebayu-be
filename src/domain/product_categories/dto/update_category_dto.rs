use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Debug, Deserialize, Serialize, Validate)]
pub struct UpdateProductCategoryRequest {
    #[validate(length(min = 1))]
    pub name: Option<String>,
}
