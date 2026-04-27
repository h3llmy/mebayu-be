use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;
 
use crate::domain::languages::entity::Language;

use crate::domain::product_categories::entity::ProductCategory;
use crate::domain::product_foundations::entity::ProductFoundation;
use crate::domain::product_materials::entity::ProductMaterial;

use crate::shared::traits::Translatable;

#[derive(Clone, Serialize, Deserialize, ToSchema)]
pub struct Product {
    pub id: Uuid,
    pub price: f64,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub category_ids: Vec<Uuid>,
    pub material_ids: Vec<Uuid>,
    pub foundation_ids: Vec<Uuid>,
    pub product_categories: Vec<ProductCategory>,
    pub product_foundations: Vec<ProductFoundation>,
    pub product_materials: Vec<ProductMaterial>,
    pub images: Vec<ProductImage>,
    pub translations: Vec<ProductTranslation>,
}

impl Translatable for Product {
    fn filter_by_language(&mut self, language_id: Uuid) {
        self.translations.retain(|t| t.language_id == language_id);

        for category in &mut self.product_categories {
            category.filter_by_language(language_id);
        }
        for foundation in &mut self.product_foundations {
            foundation.filter_by_language(language_id);
        }
        for material in &mut self.product_materials {
            material.filter_by_language(language_id);
        }
    }
}

#[derive(Clone, Serialize, Deserialize, ToSchema)]
pub struct ProductTranslation {
    pub product_id: Uuid,
    pub language_id: Uuid,
    pub language: Option<Language>,
    pub name: String,
    pub description: String,
}

#[derive(Clone, Serialize, Deserialize, Debug, ToSchema)]
pub struct ProductImage {
    pub id: Uuid,
    pub product_id: Uuid,
    pub url: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
