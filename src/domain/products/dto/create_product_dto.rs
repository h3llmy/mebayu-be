use serde::Deserialize;
use uuid::Uuid;
use validator::Validate;

#[derive(Deserialize, Validate, Debug)]
pub struct CreateProductRequest {
    #[validate(required)]
    #[validate(length(min = 1))]
    pub category_ids: Option<Vec<Uuid>>,

    #[validate(required)]
    #[validate(length(min = 1))]
    pub material_ids: Option<Vec<Uuid>>,

    #[validate(required)]
    #[validate(length(min = 1))]
    pub name: Option<String>,

    #[validate(required)]
    #[validate(length(min = 1))]
    pub material: Option<String>,

    #[validate(required)]
    #[validate(range(min = 0.01))]
    pub price: Option<f64>,

    #[validate(required)]
    #[validate(length(min = 1))]
    pub description: Option<String>,

    #[validate(required)]
    #[validate(length(min = 1))]
    pub status: Option<String>,

    #[validate(required)]
    #[validate(length(min = 1))]
    pub image_urls: Option<Vec<String>>,
}
