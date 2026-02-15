use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

use crate::domain::product_categories::entity::ProductCategory;

#[derive(Clone, Serialize, Deserialize, FromRow)]
pub struct Product {
    pub id: Uuid,
    pub category_id: Uuid,
    pub name: String,
    pub material: String,
    pub price: f64,
    pub description: String,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    #[serde(skip_deserializing)]
    #[sqlx(default)]
    pub category: Option<ProductCategory>,
}
