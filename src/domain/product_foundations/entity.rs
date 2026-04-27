use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;
 
use crate::domain::languages::entity::Language;

use crate::shared::traits::Translatable;

#[derive(Clone, Serialize, Deserialize, ToSchema)]
pub struct ProductFoundation {
    pub id: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub translations: Vec<ProductFoundationTranslation>,
}

impl Translatable for ProductFoundation {
    fn filter_by_language(&mut self, language_id: Uuid) {
        self.translations.retain(|t| t.language_id == language_id);
    }
}

#[derive(Clone, Serialize, Deserialize, ToSchema)]
pub struct ProductFoundationTranslation {
    pub foundation_id: Uuid,
    pub language_id: Uuid,
    pub language: Option<Language>,
    pub name: String,
}
