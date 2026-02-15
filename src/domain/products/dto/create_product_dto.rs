use serde::Deserialize;
use uuid::Uuid;
use validator::Validate;

#[derive(Deserialize, Validate, Debug)]
pub struct CreateProductRequest {
    #[validate(required(message = "Category ID is required"))]
    pub category_id: Option<Uuid>,

    #[validate(required(message = "Name is required"))]
    #[validate(length(min = 1, message = "Name is required"))]
    pub name: Option<String>,

    #[validate(required(message = "Material is required"))]
    #[validate(length(min = 1, message = "Material is required"))]
    pub material: Option<String>,

    #[validate(required(message = "Price is required"))]
    #[validate(range(min = 0.01, message = "Price must be greater than 0"))]
    pub price: Option<f64>,

    #[validate(required(message = "Description is required"))]
    #[validate(length(min = 1, message = "Description is required"))]
    pub description: Option<String>,

    #[validate(required(message = "Status is required"))]
    #[validate(length(min = 1, message = "Status is required"))]
    pub status: Option<String>,
}
