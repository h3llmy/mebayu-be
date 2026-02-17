use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

use crate::domain::product_categories::entity::ProductCategory;
use crate::domain::product_materials::entity::ProductMaterial;

#[derive(Clone, Serialize, Deserialize, FromRow)]
pub struct Product {
    pub id: Uuid,
    pub name: String,
    pub material: String, // Keeping this for now as it exists in DB
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
}
