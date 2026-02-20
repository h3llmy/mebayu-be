use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::domain::product_categories::entity::ProductCategory;
use crate::domain::product_materials::entity::ProductMaterial;

#[derive(Clone, Serialize, Deserialize, FromRow, ToSchema)]
pub struct Product {
    pub id: Uuid,
    pub name: String,
    pub price: f64,
    pub description: String,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    #[serde(skip_deserializing)]
    #[sqlx(skip)]
    pub category_ids: Vec<Uuid>,
    #[serde(skip_deserializing)]
    #[sqlx(skip)]
    pub material_ids: Vec<Uuid>,
    #[serde(skip_deserializing)]
    #[sqlx(default)]
    pub categories: Vec<ProductCategory>,
    #[serde(skip_deserializing)]
    #[sqlx(default)]
    pub product_materials: Vec<ProductMaterial>,
    #[serde(skip_deserializing)]
    #[sqlx(default)]
    pub images: Vec<ProductImage>,
}

#[derive(Clone, Serialize, Deserialize, FromRow, Debug, ToSchema)]
pub struct ProductImage {
    pub id: Uuid,
    pub product_id: Uuid,
    pub url: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
