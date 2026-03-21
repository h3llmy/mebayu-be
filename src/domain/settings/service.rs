use async_trait::async_trait;
use chrono::Utc;
use std::sync::Arc;
use uuid::Uuid;

use crate::{
    core::{config::Config, error::AppError},
    domain::settings::{
        dto::request::UpdateSettingRequest,
        entity::{HeroImage, Setting},
    },
};
use redis::AsyncCommands;

#[async_trait]
pub trait SettingRepository: Send + Sync {
    async fn find_first(&self) -> Result<Option<Setting>, AppError>;
    async fn create(&self, setting: &Setting) -> Result<Setting, AppError>;
    async fn update(&self, id: Uuid, setting: &Setting) -> Result<Setting, AppError>;
    async fn delete(&self, id: Uuid) -> Result<(), AppError>;
}

pub struct SettingServiceImpl {
    repository: Arc<dyn SettingRepository>,
    redis_client: redis::Client,
    config: Config,
}

const SETTING_CACHE_KEY: &str = "website_setting";

impl SettingServiceImpl {
    pub fn new(
        repository: Arc<dyn SettingRepository>,
        redis_client: redis::Client,
        config: Config,
    ) -> Self {
        Self {
            repository,
            redis_client,
            config,
        }
    }

    pub async fn get_first(&self) -> Result<Setting, AppError> {
        let mut conn = self
            .redis_client
            .get_multiplexed_async_connection()
            .await
            .map_err(|e| AppError::Internal(e.to_string()))?;

        // Try to get from cache
        let cached_setting: Option<String> = conn.get(SETTING_CACHE_KEY).await.ok();
        if let Some(cached) = cached_setting {
            if let Ok(setting) = serde_json::from_str::<Setting>(&cached) {
                return Ok(setting);
            }
        }

        // Try to get from DB
        let setting = match self.repository.find_first().await? {
            Some(s) => s,
            None => {
                // Return default setting from config
                let setting_id = Uuid::nil(); // Using nil UUID for default
                let hero_images = self
                    .config
                    .default_setting_hero_images
                    .iter()
                    .enumerate()
                    .map(|(i, url)| HeroImage {
                        id: Uuid::nil(),
                        setting_id,
                        image_url: url.clone(),
                        order_index: i as i32,
                        created_at: Utc::now(),
                        updated_at: Utc::now(),
                    })
                    .collect();

                Setting {
                    id: setting_id,
                    email: self.config.default_setting_email.clone(),
                    whatsapp_number: self.config.default_setting_whatsapp.clone(),
                    hero_images,
                    created_at: Utc::now(),
                    updated_at: Utc::now(),
                }
            }
        };

        // Cache the setting
        if let Ok(serialized) = serde_json::to_string(&setting) {
            let _: () = conn
                .set_ex(SETTING_CACHE_KEY, serialized, 3600)
                .await
                .unwrap_or_default();
        }

        Ok(setting)
    }

    pub async fn upsert(&self, req: UpdateSettingRequest) -> Result<Setting, AppError> {
        let existing = self.repository.find_first().await?;

        let setting = match existing.as_ref() {
            Some(s) => {
                // Update
                let hero_images = if let Some(images) = req.hero_images {
                    images
                        .into_iter()
                        .enumerate()
                        .map(|(i, url)| HeroImage {
                            id: Uuid::new_v4(),
                            setting_id: s.id,
                            image_url: url,
                            order_index: i as i32,
                            created_at: Utc::now(),
                            updated_at: Utc::now(),
                        })
                        .collect()
                } else {
                    s.hero_images.clone()
                };

                Setting {
                    id: s.id,
                    email: req.email.unwrap_or(s.email.clone()),
                    whatsapp_number: req.whatsapp_number.unwrap_or(s.whatsapp_number.clone()),
                    hero_images,
                    created_at: s.created_at,
                    updated_at: Utc::now(),
                }
            }
            None => {
                // Create
                let setting_id = Uuid::new_v4();
                let hero_images = req
                    .hero_images
                    .unwrap_or_default()
                    .into_iter()
                    .enumerate()
                    .map(|(i, url)| HeroImage {
                        id: Uuid::new_v4(),
                        setting_id,
                        image_url: url,
                        order_index: i as i32,
                        created_at: Utc::now(),
                        updated_at: Utc::now(),
                    })
                    .collect();

                Setting {
                    id: setting_id,
                    email: req
                        .email
                        .unwrap_or_else(|| self.config.default_setting_email.clone()),
                    whatsapp_number: req
                        .whatsapp_number
                        .unwrap_or_else(|| self.config.default_setting_whatsapp.clone()),
                    hero_images,
                    created_at: Utc::now(),
                    updated_at: Utc::now(),
                }
            }
        };

        let res = if existing.is_some() {
            self.repository.update(setting.id, &setting).await?
        } else {
            self.repository.create(&setting).await?
        };

        // Invalidate cache
        let mut conn = self
            .redis_client
            .get_multiplexed_async_connection()
            .await
            .map_err(|e| AppError::Internal(e.to_string()))?;
        let _: () = conn.del(SETTING_CACHE_KEY).await.unwrap_or_default();

        Ok(res)
    }

    pub async fn delete(&self, id: Uuid) -> Result<(), AppError> {
        self.repository.delete(id).await
    }
}
