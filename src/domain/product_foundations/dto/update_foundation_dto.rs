use serde::Deserialize;
use utoipa::ToSchema;
use validator::Validate;

use super::create_foundation_dto::ProductFoundationTranslationRequest;

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct UpdateProductFoundationRequest {
    pub translations: Option<Vec<ProductFoundationTranslationRequest>>,
}
