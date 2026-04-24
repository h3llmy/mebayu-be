use serde::Deserialize;
use utoipa::ToSchema;
use validator::Validate;

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct CreateProductFoundationRequest {
    pub translations: Vec<ProductFoundationTranslationRequest>,
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct ProductFoundationTranslationRequest {
    pub language_code: String,
    pub name: String,
}
