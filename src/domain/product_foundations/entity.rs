use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;
 
use crate::domain::languages::entity::Language;

#[derive(Clone, Serialize, Deserialize, ToSchema)]
pub struct ProductFoundation {
    pub id: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub translations: Vec<ProductFoundationTranslation>,
}

#[derive(Clone, Serialize, Deserialize, ToSchema)]
pub struct ProductFoundationTranslation {
    pub foundation_id: Uuid,
    pub language_id: Uuid,
    pub language: Option<Language>,
    pub name: String,
}
