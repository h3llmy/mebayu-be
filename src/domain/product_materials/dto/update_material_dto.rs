use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use validator::Validate;

use super::create_material_dto::ProductMaterialTranslationRequest;

#[derive(Debug, Deserialize, Serialize, Validate, ToSchema)]
pub struct UpdateProductMaterialRequest {
    pub translations: Option<Vec<ProductMaterialTranslationRequest>>,
}
