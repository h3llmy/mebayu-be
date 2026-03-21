use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Clone, Serialize, Deserialize, FromRow, ToSchema)]
pub struct Setting {
    pub id: Uuid,
    pub email: String,
    pub whatsapp_number: String,
    #[sqlx(skip)]
    pub hero_images: Vec<HeroImage>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone, Serialize, Deserialize, FromRow, ToSchema)]
pub struct HeroImage {
    pub id: Uuid,
    pub setting_id: Uuid,
    pub image_url: String,
    pub order_index: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
