use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::domain::product_categories::entity::ProductCategory;
use crate::domain::product_foundations::entity::ProductFoundation;
use crate::domain::product_materials::entity::ProductMaterial;

#[derive(Clone, Serialize, Deserialize, ToSchema)]
pub struct Product {
    pub id: Uuid,
    pub price: f64,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    #[serde(skip_deserializing)]
    pub category_ids: Vec<Uuid>,
    #[serde(skip_deserializing)]
    pub material_ids: Vec<Uuid>,
    #[serde(skip_deserializing)]
    pub foundation_ids: Vec<Uuid>,
    #[serde(skip_deserializing)]
    pub categories: Vec<ProductCategory>,
    #[serde(skip_deserializing)]
    pub product_foundations: Vec<ProductFoundation>,
    #[serde(skip_deserializing)]
    pub product_materials: Vec<ProductMaterial>,
    #[serde(skip_deserializing)]
    pub images: Vec<ProductImage>,
    #[serde(skip_deserializing)]
    pub translations: Vec<ProductTranslation>,
}

#[derive(Clone, Serialize, Deserialize, FromRow, ToSchema)]
pub struct ProductTranslation {
    pub product_id: Uuid,
    pub language_id: Uuid,
    pub name: String,
    pub description: String,
}

#[derive(Clone, Serialize, Deserialize, FromRow, Debug, ToSchema)]
pub struct ProductImage {
    pub id: Uuid,
    pub product_id: Uuid,
    pub url: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
