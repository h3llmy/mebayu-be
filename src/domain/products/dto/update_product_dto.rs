use serde::Deserialize;
use uuid::Uuid;
use validator::Validate;

#[derive(Deserialize, Validate, Debug)]
pub struct UpdateProductRequest {
    pub category_ids: Option<Vec<Uuid>>,
    pub material_ids: Option<Vec<Uuid>>,

    #[validate(length(min = 1, message = "Name is required"))]
    pub name: Option<String>,

    #[validate(length(min = 1, message = "Material is required"))]
    pub material: Option<String>,

    #[validate(range(min = 0.0, message = "Price must be greater than 0"))]
    pub price: Option<f64>,

    #[validate(length(min = 1, message = "Description is required"))]
    pub description: Option<String>,

    #[validate(length(min = 1, message = "Status is required"))]
    pub status: Option<String>,
}
