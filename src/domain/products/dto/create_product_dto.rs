use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;
use validator::Validate;

#[derive(Serialize, Deserialize, Validate, Debug, ToSchema)]
pub struct CreateProductRequest {
    #[validate(length(min = 1))]
    pub category_ids: Vec<Uuid>,

    #[validate(length(min = 1))]
    pub material_ids: Vec<Uuid>,

    #[validate(length(min = 1))]
    pub foundation_ids: Vec<Uuid>,

    #[validate(range(min = 0.01))]
    pub price: f64,

    #[validate(length(min = 1))]
    pub status: String,

    #[validate(length(min = 1))]
    pub image_urls: Vec<String>,

    #[validate(length(min = 1))]
    pub translations: Vec<ProductTranslationRequest>,
}

#[derive(Serialize, Deserialize, Validate, Debug, ToSchema)]
pub struct ProductTranslationRequest {
    pub language_id: Uuid,
    #[validate(length(min = 1))]
    pub name: String,
    #[validate(length(min = 1))]
    pub description: String,
}
