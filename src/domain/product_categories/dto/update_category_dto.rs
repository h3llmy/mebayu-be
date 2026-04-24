use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use validator::Validate;

use super::create_category_dto::ProductCategoryTranslationRequest;

#[derive(Debug, Deserialize, Serialize, Validate, ToSchema)]
pub struct UpdateProductCategoryRequest {
    pub translations: Option<Vec<ProductCategoryTranslationRequest>>,
}
