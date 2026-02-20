use serde::Deserialize;
use utoipa::ToSchema;
use uuid::Uuid;
use validator::Validate;

#[derive(Deserialize, Validate, Debug, ToSchema)]
pub struct UpdateProductRequest {
    #[validate(length(min = 1))]
    pub category_ids: Option<Vec<Uuid>>,

    #[validate(length(min = 1))]
    pub material_ids: Option<Vec<Uuid>>,

    #[validate(length(min = 1))]
    pub name: Option<String>,

    #[validate(range(min = 0.0))]
    pub price: Option<f64>,

    #[validate(length(min = 1))]
    pub description: Option<String>,

    #[validate(length(min = 1))]
    pub status: Option<String>,

    pub image_urls: Option<Vec<String>>,
}
