use async_trait::async_trait;
use chrono::Utc;
use std::sync::Arc;
use uuid::Uuid;

use crate::{
    core::{config::Config, error::AppError},
    domain::settings::{
        dto::request::UpdateSettingRequest,
        entity::{HeroImage, CraftsmanshipImage, Setting},
    },
};
use redis::AsyncCommands;

const SETTING_CACHE_KEY: &str = "website_setting";

/// Thin cache abstraction so the service can be unit-tested without Redis.
#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait SettingCache: Send + Sync {
    async fn get(&self) -> Option<Setting>;
    async fn set(&self, setting: &Setting);
    async fn invalidate(&self);
}

pub struct RedisSettingCache {
    client: redis::Client,
}

impl RedisSettingCache {
    pub fn new(client: redis::Client) -> Self {
        Self { client }
    }
}

#[async_trait]
impl SettingCache for RedisSettingCache {
    async fn get(&self) -> Option<Setting> {
        use crate::core::monitoring::observe_redis;
        let mut conn = self.client.get_multiplexed_async_connection().await.ok()?;
        let cached: Option<String> = observe_redis("get_setting", conn.get(SETTING_CACHE_KEY))
            .await
            .ok()?;
        cached.and_then(|s| serde_json::from_str(&s).ok())
    }

    async fn set(&self, setting: &Setting) {
        use crate::core::monitoring::observe_redis;
        if let Ok(mut conn) = self.client.get_multiplexed_async_connection().await {
            if let Ok(serialized) = serde_json::to_string(setting) {
                let _: Result<(), _> =
                    observe_redis("set_setting", conn.set_ex(SETTING_CACHE_KEY, serialized, 3600))
                        .await;
            }
        }
    }

    async fn invalidate(&self) {
        if let Ok(mut conn) = self.client.get_multiplexed_async_connection().await {
            let _: () = conn.del(SETTING_CACHE_KEY).await.unwrap_or_default();
        }
    }
}

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait SettingRepository: Send + Sync {
    async fn find_first(&self) -> Result<Option<Setting>, AppError>;
    async fn create(&self, setting: &Setting) -> Result<Setting, AppError>;
    async fn update(&self, id: Uuid, setting: &Setting) -> Result<Setting, AppError>;
    async fn delete(&self, id: Uuid) -> Result<(), AppError>;
}

pub struct SettingServiceImpl {
    repository: Arc<dyn SettingRepository>,
    cache: Arc<dyn SettingCache>,
    config: Config,
}

impl SettingServiceImpl {
    pub fn new(
        repository: Arc<dyn SettingRepository>,
        cache: Arc<dyn SettingCache>,
        config: Config,
    ) -> Self {
        Self {
            repository,
            cache,
            config,
        }
    }

    pub async fn get_first(&self) -> Result<Setting, AppError> {
        // Try cache first
        if let Some(cached) = self.cache.get().await {
            return Ok(cached);
        }

        // Fall back to DB
        let setting = match self.repository.find_first().await? {
            Some(s) => s,
            None => {
                let setting_id = Uuid::nil();
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
                    translations: vec![],
                    craftsmanship_images: vec![],
                    created_at: Utc::now(),
                    updated_at: Utc::now(),
                }
            }
        };

        self.cache.set(&setting).await;
        Ok(setting)
    }

    pub async fn upsert(&self, req: UpdateSettingRequest) -> Result<Setting, AppError> {
        let existing = self.repository.find_first().await?;

        let setting = match existing.as_ref() {
            Some(s) => {
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

                let translations = if let Some(trs) = &req.translations {
                    trs.iter()
                        .map(|t| crate::domain::settings::entity::SettingTranslation {
                            setting_id: s.id,
                            language_id: t.language_id,
                            hero_title: t.hero_title.clone(),
                            hero_description: t.hero_description.clone(),
                            about_title: t.about_title.clone(),
                            about_description: t.about_description.clone(),
                            about_image_url: t.about_image_url.clone(),
                        })
                        .collect()
                } else {
                    s.translations.clone()
                };

                let craftsmanship_images = if let Some(images) = req.craftsmanship_image_urls {
                    images
                        .into_iter()
                        .enumerate()
                        .map(|(i, url)| CraftsmanshipImage {
                            id: Uuid::new_v4(),
                            setting_id: s.id,
                            image_url: url,
                            order_index: i as i32,
                            created_at: Utc::now(),
                            updated_at: Utc::now(),
                        })
                        .collect()
                } else {
                    s.craftsmanship_images.clone()
                };

                Setting {
                    id: s.id,
                    email: req.email.unwrap_or(s.email.clone()),
                    whatsapp_number: req.whatsapp_number.unwrap_or(s.whatsapp_number.clone()),
                    hero_images,
                    translations,
                    craftsmanship_images,
                    created_at: s.created_at,
                    updated_at: Utc::now(),
                }
            }
            None => {
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

                let translations = req
                    .translations
                    .unwrap_or_default()
                    .into_iter()
                    .map(|t| crate::domain::settings::entity::SettingTranslation {
                        setting_id,
                        language_id: t.language_id,
                        hero_title: t.hero_title,
                        hero_description: t.hero_description,
                        about_title: t.about_title,
                        about_description: t.about_description,
                        about_image_url: t.about_image_url,
                    })
                    .collect();

                let craftsmanship_images = req
                    .craftsmanship_image_urls
                    .unwrap_or_default()
                    .into_iter()
                    .enumerate()
                    .map(|(i, url)| CraftsmanshipImage {
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
                    translations,
                    craftsmanship_images,
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

        self.cache.invalidate().await;
        Ok(res)
    }

    pub async fn delete(&self, id: Uuid) -> Result<(), AppError> {
        self.repository.delete(id).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::settings::{
        dto::request::SettingTranslationRequest,
        entity::SettingTranslation,
    };

    fn make_config() -> Config {
        Config {
            default_setting_email: "default@mebayu.com".to_string(),
            default_setting_whatsapp: "628000000000".to_string(),
            default_setting_hero_images: vec!["http://img1.jpg".to_string(), "http://img2.jpg".to_string()],
            ..Config::default()
        }
    }

    fn make_setting(id: Uuid) -> Setting {
        Setting {
            id,
            email: "existing@mebayu.com".to_string(),
            whatsapp_number: "+62812345".to_string(),
            hero_images: vec![],
            translations: vec![],
            craftsmanship_images: vec![],
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    // --- get_first tests ---

    #[tokio::test]
    async fn test_get_first_returns_cached_when_cache_hit() {
        let id = Uuid::new_v4();
        let cached = make_setting(id);
        let cached_clone = cached.clone();

        let mut mock_cache = MockSettingCache::new();
        mock_cache
            .expect_get()
            .times(1)
            .returning(move || Some(cached_clone.clone()));

        // Repository must NOT be called on a cache hit
        let mut mock_repo = MockSettingRepository::new();
        mock_repo.expect_find_first().times(0);

        let service = SettingServiceImpl::new(Arc::new(mock_repo), Arc::new(mock_cache), make_config());
        let result = service.get_first().await.unwrap();

        assert_eq!(result.id, id);
        assert_eq!(result.email, "existing@mebayu.com");
    }

    #[tokio::test]
    async fn test_get_first_falls_through_to_db_on_cache_miss() {
        let id = Uuid::new_v4();
        let db_setting = make_setting(id);
        let db_setting_clone = db_setting.clone();

        let mut mock_cache = MockSettingCache::new();
        mock_cache.expect_get().times(1).returning(|| None);
        mock_cache.expect_set().times(1).returning(|_| ());

        let mut mock_repo = MockSettingRepository::new();
        mock_repo
            .expect_find_first()
            .times(1)
            .returning(move || Ok(Some(db_setting_clone.clone())));

        let service = SettingServiceImpl::new(Arc::new(mock_repo), Arc::new(mock_cache), make_config());
        let result = service.get_first().await.unwrap();

        assert_eq!(result.id, id);
    }

    #[tokio::test]
    async fn test_get_first_returns_default_when_db_empty() {
        let mut mock_cache = MockSettingCache::new();
        mock_cache.expect_get().times(1).returning(|| None);
        mock_cache.expect_set().times(1).returning(|_| ());

        let mut mock_repo = MockSettingRepository::new();
        mock_repo
            .expect_find_first()
            .times(1)
            .returning(|| Ok(None));

        let config = make_config();
        let service = SettingServiceImpl::new(Arc::new(mock_repo), Arc::new(mock_cache), config.clone());
        let result = service.get_first().await.unwrap();

        assert_eq!(result.id, Uuid::nil());
        assert_eq!(result.email, config.default_setting_email);
        assert_eq!(result.whatsapp_number, config.default_setting_whatsapp);
        assert_eq!(result.hero_images.len(), config.default_setting_hero_images.len());
    }

    // --- upsert (create path) tests ---

    #[tokio::test]
    async fn test_upsert_creates_setting_when_none_exists() {
        let mut mock_cache = MockSettingCache::new();
        mock_cache.expect_invalidate().times(1).returning(|| ());

        let mut mock_repo = MockSettingRepository::new();
        mock_repo.expect_find_first().times(1).returning(|| Ok(None));
        mock_repo
            .expect_create()
            .times(1)
            .returning(|s| Ok(s.clone()));

        let req = UpdateSettingRequest {
            email: Some("new@mebayu.com".to_string()),
            whatsapp_number: Some("+62899999".to_string()),
            hero_images: None,
            craftsmanship_image_urls: None,
            translations: None,
        };

        let service = SettingServiceImpl::new(Arc::new(mock_repo), Arc::new(mock_cache), make_config());
        let result = service.upsert(req).await.unwrap();

        assert_eq!(result.email, "new@mebayu.com");
        assert_eq!(result.whatsapp_number, "+62899999");
        assert!(result.hero_images.is_empty());
        assert!(result.craftsmanship_images.is_empty());
    }

    #[tokio::test]
    async fn test_upsert_create_uses_config_defaults_when_fields_absent() {
        let mut mock_cache = MockSettingCache::new();
        mock_cache.expect_invalidate().times(1).returning(|| ());

        let mut mock_repo = MockSettingRepository::new();
        mock_repo.expect_find_first().times(1).returning(|| Ok(None));
        mock_repo
            .expect_create()
            .times(1)
            .returning(|s| Ok(s.clone()));

        let req = UpdateSettingRequest {
            email: None,
            whatsapp_number: None,
            hero_images: None,
            craftsmanship_image_urls: None,
            translations: None,
        };

        let config = make_config();
        let service = SettingServiceImpl::new(Arc::new(mock_repo), Arc::new(mock_cache), config.clone());
        let result = service.upsert(req).await.unwrap();

        assert_eq!(result.email, config.default_setting_email);
        assert_eq!(result.whatsapp_number, config.default_setting_whatsapp);
    }

    #[tokio::test]
    async fn test_upsert_create_with_hero_images() {
        let mut mock_cache = MockSettingCache::new();
        mock_cache.expect_invalidate().times(1).returning(|| ());

        let mut mock_repo = MockSettingRepository::new();
        mock_repo.expect_find_first().times(1).returning(|| Ok(None));
        mock_repo
            .expect_create()
            .times(1)
            .returning(|s| Ok(s.clone()));

        let req = UpdateSettingRequest {
            email: Some("test@test.com".to_string()),
            whatsapp_number: None,
            hero_images: Some(vec!["http://hero1.jpg".to_string(), "http://hero2.jpg".to_string()]),
            craftsmanship_image_urls: None,
            translations: None,
        };

        let service = SettingServiceImpl::new(Arc::new(mock_repo), Arc::new(mock_cache), make_config());
        let result = service.upsert(req).await.unwrap();

        assert_eq!(result.hero_images.len(), 2);
        assert_eq!(result.hero_images[0].image_url, "http://hero1.jpg");
        assert_eq!(result.hero_images[0].order_index, 0);
        assert_eq!(result.hero_images[1].order_index, 1);
    }

    #[tokio::test]
    async fn test_upsert_create_with_craftsmanship_images() {
        let mut mock_cache = MockSettingCache::new();
        mock_cache.expect_invalidate().times(1).returning(|| ());

        let mut mock_repo = MockSettingRepository::new();
        mock_repo.expect_find_first().times(1).returning(|| Ok(None));
        mock_repo
            .expect_create()
            .times(1)
            .returning(|s| Ok(s.clone()));

        let req = UpdateSettingRequest {
            email: Some("test@test.com".to_string()),
            whatsapp_number: None,
            hero_images: None,
            craftsmanship_image_urls: Some(vec![
                "http://c1.jpg".to_string(),
                "http://c2.jpg".to_string(),
                "http://c3.jpg".to_string(),
                "http://c4.jpg".to_string(),
            ]),
            translations: None,
        };

        let service = SettingServiceImpl::new(Arc::new(mock_repo), Arc::new(mock_cache), make_config());
        let result = service.upsert(req).await.unwrap();

        assert_eq!(result.craftsmanship_images.len(), 4);
        for (i, img) in result.craftsmanship_images.iter().enumerate() {
            assert_eq!(img.order_index, i as i32);
        }
    }

    #[tokio::test]
    async fn test_upsert_create_with_translations() {
        let lang_id = Uuid::new_v4();

        let mut mock_cache = MockSettingCache::new();
        mock_cache.expect_invalidate().times(1).returning(|| ());

        let mut mock_repo = MockSettingRepository::new();
        mock_repo.expect_find_first().times(1).returning(|| Ok(None));
        mock_repo
            .expect_create()
            .times(1)
            .returning(|s| Ok(s.clone()));

        let req = UpdateSettingRequest {
            email: None,
            whatsapp_number: None,
            hero_images: None,
            craftsmanship_image_urls: None,
            translations: Some(vec![SettingTranslationRequest {
                language_id: lang_id,
                hero_title: "Welcome".to_string(),
                hero_description: "Best crafts".to_string(),
                about_title: "About us".to_string(),
                about_description: "We are mebayu".to_string(),
                about_image_url: "http://about.jpg".to_string(),
            }]),
        };

        let service = SettingServiceImpl::new(Arc::new(mock_repo), Arc::new(mock_cache), make_config());
        let result = service.upsert(req).await.unwrap();

        assert_eq!(result.translations.len(), 1);
        assert_eq!(result.translations[0].language_id, lang_id);
        assert_eq!(result.translations[0].hero_title, "Welcome");
    }

    // --- upsert (update path) tests ---

    #[tokio::test]
    async fn test_upsert_updates_existing_setting() {
        let id = Uuid::new_v4();
        let existing = make_setting(id);
        let existing_clone = existing.clone();

        let mut mock_cache = MockSettingCache::new();
        mock_cache.expect_invalidate().times(1).returning(|| ());

        let mut mock_repo = MockSettingRepository::new();
        mock_repo
            .expect_find_first()
            .times(1)
            .returning(move || Ok(Some(existing_clone.clone())));
        mock_repo
            .expect_update()
            .times(1)
            .returning(|_, s| Ok(s.clone()));

        let req = UpdateSettingRequest {
            email: Some("updated@mebayu.com".to_string()),
            whatsapp_number: Some("+62899999".to_string()),
            hero_images: None,
            craftsmanship_image_urls: None,
            translations: None,
        };

        let service = SettingServiceImpl::new(Arc::new(mock_repo), Arc::new(mock_cache), make_config());
        let result = service.upsert(req).await.unwrap();

        assert_eq!(result.id, id);
        assert_eq!(result.email, "updated@mebayu.com");
    }

    #[tokio::test]
    async fn test_upsert_update_preserves_fields_when_none() {
        let id = Uuid::new_v4();
        let lang_id = Uuid::new_v4();
        let mut existing = make_setting(id);
        existing.hero_images = vec![HeroImage {
            id: Uuid::new_v4(),
            setting_id: id,
            image_url: "http://old.jpg".to_string(),
            order_index: 0,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }];
        existing.translations = vec![SettingTranslation {
            setting_id: id,
            language_id: lang_id,
            hero_title: "Old Title".to_string(),
            hero_description: "Old Desc".to_string(),
            about_title: "Old About".to_string(),
            about_description: "Old About Desc".to_string(),
            about_image_url: "http://old-about.jpg".to_string(),
        }];
        let existing_clone = existing.clone();

        let mut mock_cache = MockSettingCache::new();
        mock_cache.expect_invalidate().times(1).returning(|| ());

        let mut mock_repo = MockSettingRepository::new();
        mock_repo
            .expect_find_first()
            .times(1)
            .returning(move || Ok(Some(existing_clone.clone())));
        mock_repo
            .expect_update()
            .times(1)
            .returning(|_, s| Ok(s.clone()));

        // Only email is changed; hero_images, translations, craftsmanship_images are None → keep old
        let req = UpdateSettingRequest {
            email: Some("partial@mebayu.com".to_string()),
            whatsapp_number: None,
            hero_images: None,
            craftsmanship_image_urls: None,
            translations: None,
        };

        let service = SettingServiceImpl::new(Arc::new(mock_repo), Arc::new(mock_cache), make_config());
        let result = service.upsert(req).await.unwrap();

        assert_eq!(result.email, "partial@mebayu.com");
        assert_eq!(result.whatsapp_number, existing.whatsapp_number);
        assert_eq!(result.hero_images.len(), 1);
        assert_eq!(result.hero_images[0].image_url, "http://old.jpg");
        assert_eq!(result.translations.len(), 1);
        assert_eq!(result.translations[0].hero_title, "Old Title");
    }

    #[tokio::test]
    async fn test_upsert_update_replaces_hero_images_when_provided() {
        let id = Uuid::new_v4();
        let mut existing = make_setting(id);
        existing.hero_images = vec![HeroImage {
            id: Uuid::new_v4(),
            setting_id: id,
            image_url: "http://old.jpg".to_string(),
            order_index: 0,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }];
        let existing_clone = existing.clone();

        let mut mock_cache = MockSettingCache::new();
        mock_cache.expect_invalidate().times(1).returning(|| ());

        let mut mock_repo = MockSettingRepository::new();
        mock_repo
            .expect_find_first()
            .times(1)
            .returning(move || Ok(Some(existing_clone.clone())));
        mock_repo
            .expect_update()
            .times(1)
            .returning(|_, s| Ok(s.clone()));

        let req = UpdateSettingRequest {
            email: None,
            whatsapp_number: None,
            hero_images: Some(vec!["http://new1.jpg".to_string(), "http://new2.jpg".to_string()]),
            craftsmanship_image_urls: None,
            translations: None,
        };

        let service = SettingServiceImpl::new(Arc::new(mock_repo), Arc::new(mock_cache), make_config());
        let result = service.upsert(req).await.unwrap();

        assert_eq!(result.hero_images.len(), 2);
        assert_eq!(result.hero_images[0].image_url, "http://new1.jpg");
        assert_eq!(result.hero_images[1].image_url, "http://new2.jpg");
    }

    // --- delete tests ---

    #[tokio::test]
    async fn test_delete_delegates_to_repository() {
        let id = Uuid::new_v4();

        let mock_cache = MockSettingCache::new();
        let mut mock_repo = MockSettingRepository::new();
        mock_repo
            .expect_delete()
            .with(mockall::predicate::eq(id))
            .times(1)
            .returning(|_| Ok(()));

        let service = SettingServiceImpl::new(Arc::new(mock_repo), Arc::new(mock_cache), make_config());
        let result = service.delete(id).await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_delete_propagates_not_found_error() {
        let id = Uuid::new_v4();

        let mock_cache = MockSettingCache::new();
        let mut mock_repo = MockSettingRepository::new();
        mock_repo
            .expect_delete()
            .times(1)
            .returning(|_| Err(AppError::NotFound("Setting not found".to_string())));

        let service = SettingServiceImpl::new(Arc::new(mock_repo), Arc::new(mock_cache), make_config());
        let result = service.delete(id).await;

        assert!(matches!(result, Err(AppError::NotFound(_))));
    }
}
