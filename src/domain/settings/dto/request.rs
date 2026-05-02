use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use validator::Validate;

#[derive(Clone, Serialize, Deserialize, ToSchema, Validate)]
pub struct CreateSettingRequest {
    pub email: String,
    pub whatsapp_number: String,
    pub hero_images: Vec<String>,
}

#[derive(Clone, Serialize, Deserialize, ToSchema, Validate)]
pub struct UpdateSettingRequest {
    pub email: Option<String>,
    pub whatsapp_number: Option<String>,
    pub hero_images: Option<Vec<String>>,
    pub translations: Option<Vec<SettingTranslationRequest>>,
}

#[derive(Clone, Serialize, Deserialize, ToSchema, Validate)]
pub struct SettingTranslationRequest {
    pub language_id: uuid::Uuid,
    pub hero_title: String,
    pub hero_description: String,
    pub about_title: String,
    pub about_description: String,
    pub about_image_url: String,
}
