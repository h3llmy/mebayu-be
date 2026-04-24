use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use validator::Validate;

#[derive(Debug, Deserialize, Serialize, Validate, ToSchema)]
pub struct CreateProductMaterialRequest {
    pub translations: Vec<ProductMaterialTranslationRequest>,
}

#[derive(Debug, Deserialize, Serialize, Validate, ToSchema)]
pub struct ProductMaterialTranslationRequest {
    pub language_code: String,
    pub name: String,
}
