use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;
 
use crate::domain::languages::entity::Language;

#[derive(Clone, Serialize, Deserialize, ToSchema)]
pub struct ProductMaterial {
    pub id: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub translations: Vec<ProductMaterialTranslation>,
}

#[derive(Clone, Serialize, Deserialize, ToSchema)]
pub struct ProductMaterialTranslation {
    pub material_id: Uuid,
    pub language_id: Uuid,
    pub language: Option<Language>,
    pub name: String,
}
