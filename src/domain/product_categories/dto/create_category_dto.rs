use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use validator::Validate;

#[derive(Debug, Deserialize, Serialize, Validate, ToSchema)]
pub struct CreateProductCategoryRequest {
    pub translations: Vec<ProductCategoryTranslationRequest>,
}

#[derive(Debug, Deserialize, Serialize, Validate, ToSchema)]
pub struct ProductCategoryTranslationRequest {
    pub language_code: String,
    pub name: String,
}
