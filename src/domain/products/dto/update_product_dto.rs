use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;
use validator::Validate;

use super::create_product_dto::ProductTranslationRequest;

#[derive(Serialize, Deserialize, Validate, Debug, ToSchema)]
pub struct UpdateProductRequest {
    #[validate(length(min = 1))]
    pub category_ids: Option<Vec<Uuid>>,

    #[validate(length(min = 1))]
    pub material_ids: Option<Vec<Uuid>>,

    #[validate(length(min = 1))]
    pub foundation_ids: Option<Vec<Uuid>>,

    #[validate(range(min = 0.0))]
    pub price: Option<f64>,

    #[validate(length(min = 1))]
    pub status: Option<String>,

    pub image_urls: Option<Vec<String>>,

    pub translations: Option<Vec<ProductTranslationRequest>>,
}
