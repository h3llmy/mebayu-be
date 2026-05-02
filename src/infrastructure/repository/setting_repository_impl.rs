use async_trait::async_trait;
use chrono::Utc;
use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    core::error::AppError,
    domain::settings::{
        entity::{HeroImage, Setting},
        service::SettingRepository,
    },
};

pub struct SettingRepositoryImpl {
    pool: PgPool,
}

impl SettingRepositoryImpl {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl SettingRepository for SettingRepositoryImpl {
    async fn find_first(&self) -> Result<Option<Setting>, AppError> {
        let setting = sqlx::query_as::<_, Setting>(
            "SELECT id, email, whatsapp_number, created_at, updated_at FROM settings LIMIT 1",
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e: sqlx::Error| AppError::Database(e.to_string()))?;

        if let Some(mut s) = setting {
            let images = sqlx::query_as!(
                HeroImage,
                "SELECT * FROM hero_images WHERE setting_id = $1 ORDER BY order_index",
                s.id
            )
            .fetch_all(&self.pool)
            .await
            .map_err(|e: sqlx::Error| AppError::Database(e.to_string()))?;
            s.hero_images = images;

            let translations = sqlx::query_as!(
                crate::domain::settings::entity::SettingTranslation,
                "SELECT setting_id, language_id, hero_title, hero_description, about_title, about_description, about_image_url FROM setting_translations WHERE setting_id = $1",
                s.id
            )
            .fetch_all(&self.pool)
            .await
            .map_err(|e: sqlx::Error| AppError::Database(e.to_string()))?;
            s.translations = translations;

            let craft_images = sqlx::query_as!(
                crate::domain::settings::entity::CraftsmanshipImage,
                "SELECT * FROM craftsmanship_images WHERE setting_id = $1 ORDER BY order_index",
                s.id
            )
            .fetch_all(&self.pool)
            .await
            .map_err(|e: sqlx::Error| AppError::Database(e.to_string()))?;
            s.craftsmanship_images = craft_images;

            Ok(Some(s))
        } else {
            Ok(None)
        }
    }

    async fn create(&self, setting: &Setting) -> Result<Setting, AppError> {
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e: sqlx::Error| AppError::Database(e.to_string()))?;

        let mut setting_res = sqlx::query_as::<_, Setting>(
            r#"
            INSERT INTO settings (id, email, whatsapp_number, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING id, email, whatsapp_number, created_at, updated_at
            "#,
        )
        .bind(setting.id)
        .bind(&setting.email)
        .bind(&setting.whatsapp_number)
        .bind(setting.created_at)
        .bind(setting.updated_at)
        .fetch_one(&mut *tx)
        .await
        .map_err(|e: sqlx::Error| AppError::Database(e.to_string()))?;

        let mut created_images = Vec::new();
        for image in &setting.hero_images {
            let res = sqlx::query_as!(
                HeroImage,
                r#"
                INSERT INTO hero_images (id, setting_id, image_url, order_index, created_at, updated_at)
                VALUES ($1, $2, $3, $4, $5, $6)
                RETURNING *
                "#,
                Uuid::new_v4(),
                setting_res.id,
                image.image_url,
                image.order_index,
                setting.created_at,
                setting.updated_at
            )
            .fetch_one(&mut *tx)
            .await
            .map_err(|e: sqlx::Error| AppError::Database(e.to_string()))?;
            created_images.push(res);
        }

        for tr in &setting.translations {
            sqlx::query!(
                r#"
                INSERT INTO setting_translations (setting_id, language_id, hero_title, hero_description, about_title, about_description, about_image_url)
                VALUES ($1, $2, $3, $4, $5, $6, $7)
                "#,
                setting_res.id,
                tr.language_id,
                tr.hero_title,
                tr.hero_description,
                tr.about_title,
                tr.about_description,
                tr.about_image_url
            )
            .execute(&mut *tx)
            .await
            .map_err(|e: sqlx::Error| AppError::Database(e.to_string()))?;
        }

        let mut created_craft_images = Vec::new();
        for image in &setting.craftsmanship_images {
            let res = sqlx::query_as!(
                crate::domain::settings::entity::CraftsmanshipImage,
                r#"
                INSERT INTO craftsmanship_images (id, setting_id, image_url, order_index, created_at, updated_at)
                VALUES ($1, $2, $3, $4, $5, $6)
                RETURNING *
                "#,
                Uuid::new_v4(),
                setting_res.id,
                image.image_url,
                image.order_index,
                setting.created_at,
                setting.updated_at
            )
            .fetch_one(&mut *tx)
            .await
            .map_err(|e: sqlx::Error| AppError::Database(e.to_string()))?;
            created_craft_images.push(res);
        }

        tx.commit()
            .await
            .map_err(|e: sqlx::Error| AppError::Database(e.to_string()))?;
        setting_res.hero_images = created_images;
        setting_res.translations = setting.translations.clone();
        setting_res.craftsmanship_images = created_craft_images;
        Ok(setting_res)
    }

    async fn update(&self, id: Uuid, setting: &Setting) -> Result<Setting, AppError> {
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e: sqlx::Error| AppError::Database(e.to_string()))?;

        let mut setting_res = sqlx::query_as::<_, Setting>(
            r#"
            UPDATE settings 
            SET email = $2, whatsapp_number = $3, updated_at = $4
            WHERE id = $1
            RETURNING id, email, whatsapp_number, created_at, updated_at
            "#,
        )
        .bind(id)
        .bind(&setting.email)
        .bind(&setting.whatsapp_number)
        .bind(setting.updated_at)
        .fetch_optional(&mut *tx)
        .await
        .map_err(|e: sqlx::Error| AppError::Database(e.to_string()))?
        .ok_or_else(|| AppError::NotFound("Setting not found".to_string()))?;

        // Delete old images
        sqlx::query!("DELETE FROM hero_images WHERE setting_id = $1", id)
            .execute(&mut *tx)
            .await
            .map_err(|e: sqlx::Error| AppError::Database(e.to_string()))?;

        // Insert new images
        let mut created_images = Vec::new();
        for image in &setting.hero_images {
            let res = sqlx::query_as!(
                HeroImage,
                r#"
                INSERT INTO hero_images (id, setting_id, image_url, order_index, created_at, updated_at)
                VALUES ($1, $2, $3, $4, $5, $6)
                RETURNING *
                "#,
                Uuid::new_v4(),
                id,
                image.image_url,
                image.order_index,
                Utc::now(),
                Utc::now()
            )
            .fetch_one(&mut *tx)
            .await
            .map_err(|e: sqlx::Error| AppError::Database(e.to_string()))?;
            created_images.push(res);
        }

        sqlx::query!("DELETE FROM setting_translations WHERE setting_id = $1", id)
            .execute(&mut *tx)
            .await
            .map_err(|e: sqlx::Error| AppError::Database(e.to_string()))?;

        for tr in &setting.translations {
            sqlx::query!(
                r#"
                INSERT INTO setting_translations (setting_id, language_id, hero_title, hero_description, about_title, about_description, about_image_url)
                VALUES ($1, $2, $3, $4, $5, $6, $7)
                "#,
                id,
                tr.language_id,
                tr.hero_title,
                tr.hero_description,
                tr.about_title,
                tr.about_description,
                tr.about_image_url
            )
            .execute(&mut *tx)
            .await
            .map_err(|e: sqlx::Error| AppError::Database(e.to_string()))?;
        }

        sqlx::query!("DELETE FROM craftsmanship_images WHERE setting_id = $1", id)
            .execute(&mut *tx)
            .await
            .map_err(|e: sqlx::Error| AppError::Database(e.to_string()))?;

        let mut created_craft_images = Vec::new();
        for image in &setting.craftsmanship_images {
            let res = sqlx::query_as!(
                crate::domain::settings::entity::CraftsmanshipImage,
                r#"
                INSERT INTO craftsmanship_images (id, setting_id, image_url, order_index, created_at, updated_at)
                VALUES ($1, $2, $3, $4, $5, $6)
                RETURNING *
                "#,
                Uuid::new_v4(),
                id,
                image.image_url,
                image.order_index,
                Utc::now(),
                Utc::now()
            )
            .fetch_one(&mut *tx)
            .await
            .map_err(|e: sqlx::Error| AppError::Database(e.to_string()))?;
            created_craft_images.push(res);
        }

        tx.commit()
            .await
            .map_err(|e: sqlx::Error| AppError::Database(e.to_string()))?;
        setting_res.hero_images = created_images;
        setting_res.translations = setting.translations.clone();
        setting_res.craftsmanship_images = created_craft_images;
        Ok(setting_res)
    }

    async fn delete(&self, id: Uuid) -> Result<(), AppError> {
        let result = sqlx::query!("DELETE FROM settings WHERE id = $1", id)
            .execute(&self.pool)
            .await
            .map_err(|e: sqlx::Error| AppError::Database(e.to_string()))?;

        if result.rows_affected() == 0 {
            return Err(AppError::NotFound("Setting not found".to_string()));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        domain::settings::entity::{CraftsmanshipImage, HeroImage, Setting, SettingTranslation},
        infrastructure::database::migrations::run_migrations,
    };
    use chrono::Utc;
    use sqlx::PgPool;

    async fn setup(pool: &PgPool) {
        run_migrations(pool).await;
        // Clean any leftover data from previous runs
        sqlx::query!("DELETE FROM settings").execute(pool).await.ok();
    }

    fn make_setting() -> Setting {
        let id = Uuid::new_v4();
        Setting {
            id,
            email: "test@mebayu.com".to_string(),
            whatsapp_number: "+62812345".to_string(),
            hero_images: vec![],
            translations: vec![],
            craftsmanship_images: vec![],
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    // Helper: insert a language so foreign key constraints are satisfied.
    // Uses a unique code per call to avoid conflicts in concurrent test runs.
    // "l" + first 7 hex chars of UUID = 8 chars, within VARCHAR(10).
    async fn insert_language(pool: &PgPool) -> Uuid {
        let lang_id = Uuid::new_v4();
        let unique_code = format!("l{}", &lang_id.simple().to_string()[..7]);
        sqlx::query!(
            "INSERT INTO languages (id, code, name, is_default) VALUES ($1, $2, $3, $4)",
            lang_id,
            unique_code,
            "English",
            true,
        )
        .execute(pool)
        .await
        .unwrap();
        lang_id
    }

    #[sqlx::test]
    async fn test_find_first_returns_none_when_empty(pool: PgPool) {
        setup(&pool).await;
        let repo = SettingRepositoryImpl::new(pool.clone());

        let result = repo.find_first().await.unwrap();
        assert!(result.is_none());
    }

    #[sqlx::test]
    async fn test_create_and_find_first(pool: PgPool) {
        setup(&pool).await;
        let repo = SettingRepositoryImpl::new(pool.clone());

        let setting = make_setting();
        let created = repo.create(&setting).await.unwrap();

        assert_eq!(created.id, setting.id);
        assert_eq!(created.email, "test@mebayu.com");
        assert!(created.hero_images.is_empty());
        assert!(created.translations.is_empty());
        assert!(created.craftsmanship_images.is_empty());

        let found = repo.find_first().await.unwrap();
        assert!(found.is_some());
        assert_eq!(found.unwrap().id, setting.id);
    }

    #[sqlx::test]
    async fn test_create_with_hero_images(pool: PgPool) {
        setup(&pool).await;
        let repo = SettingRepositoryImpl::new(pool.clone());

        let id = Uuid::new_v4();
        let setting = Setting {
            id,
            hero_images: vec![
                HeroImage { id: Uuid::new_v4(), setting_id: id, image_url: "http://h1.jpg".to_string(), order_index: 0, created_at: Utc::now(), updated_at: Utc::now() },
                HeroImage { id: Uuid::new_v4(), setting_id: id, image_url: "http://h2.jpg".to_string(), order_index: 1, created_at: Utc::now(), updated_at: Utc::now() },
            ],
            ..make_setting()
        };

        let created = repo.create(&setting).await.unwrap();
        assert_eq!(created.hero_images.len(), 2);
        assert_eq!(created.hero_images[0].image_url, "http://h1.jpg");
        assert_eq!(created.hero_images[1].image_url, "http://h2.jpg");
    }

    #[sqlx::test]
    async fn test_create_with_craftsmanship_images(pool: PgPool) {
        setup(&pool).await;
        let repo = SettingRepositoryImpl::new(pool.clone());

        let id = Uuid::new_v4();
        let setting = Setting {
            id,
            craftsmanship_images: vec![
                CraftsmanshipImage { id: Uuid::new_v4(), setting_id: id, image_url: "http://c1.jpg".to_string(), order_index: 0, created_at: Utc::now(), updated_at: Utc::now() },
                CraftsmanshipImage { id: Uuid::new_v4(), setting_id: id, image_url: "http://c2.jpg".to_string(), order_index: 1, created_at: Utc::now(), updated_at: Utc::now() },
            ],
            ..make_setting()
        };

        let created = repo.create(&setting).await.unwrap();
        assert_eq!(created.craftsmanship_images.len(), 2);
        assert_eq!(created.craftsmanship_images[0].image_url, "http://c1.jpg");
    }

    #[sqlx::test]
    async fn test_create_with_translations(pool: PgPool) {
        setup(&pool).await;
        let repo = SettingRepositoryImpl::new(pool.clone());
        let lang_id = insert_language(&pool).await;

        let id = Uuid::new_v4();
        let setting = Setting {
            id,
            translations: vec![SettingTranslation {
                setting_id: id,
                language_id: lang_id,
                hero_title: "Welcome".to_string(),
                hero_description: "Best crafts".to_string(),
                about_title: "About".to_string(),
                about_description: "About us".to_string(),
                about_image_url: "http://about.jpg".to_string(),
            }],
            ..make_setting()
        };

        let created = repo.create(&setting).await.unwrap();
        assert_eq!(created.translations.len(), 1);
        assert_eq!(created.translations[0].hero_title, "Welcome");
        assert_eq!(created.translations[0].language_id, lang_id);
    }

    #[sqlx::test]
    async fn test_update_email_and_whatsapp(pool: PgPool) {
        setup(&pool).await;
        let repo = SettingRepositoryImpl::new(pool.clone());

        let setting = make_setting();
        repo.create(&setting).await.unwrap();

        let updated = Setting {
            id: setting.id,
            email: "updated@mebayu.com".to_string(),
            whatsapp_number: "+62899999".to_string(),
            hero_images: vec![],
            translations: vec![],
            craftsmanship_images: vec![],
            created_at: setting.created_at,
            updated_at: Utc::now(),
        };

        let result = repo.update(setting.id, &updated).await.unwrap();
        assert_eq!(result.email, "updated@mebayu.com");
        assert_eq!(result.whatsapp_number, "+62899999");
    }

    #[sqlx::test]
    async fn test_update_replaces_hero_images(pool: PgPool) {
        setup(&pool).await;
        let repo = SettingRepositoryImpl::new(pool.clone());

        let id = Uuid::new_v4();
        let setting = Setting {
            id,
            hero_images: vec![HeroImage { id: Uuid::new_v4(), setting_id: id, image_url: "http://old.jpg".to_string(), order_index: 0, created_at: Utc::now(), updated_at: Utc::now() }],
            ..make_setting()
        };
        repo.create(&setting).await.unwrap();

        let updated = Setting {
            id,
            email: setting.email.clone(),
            whatsapp_number: setting.whatsapp_number.clone(),
            hero_images: vec![
                HeroImage { id: Uuid::new_v4(), setting_id: id, image_url: "http://new1.jpg".to_string(), order_index: 0, created_at: Utc::now(), updated_at: Utc::now() },
                HeroImage { id: Uuid::new_v4(), setting_id: id, image_url: "http://new2.jpg".to_string(), order_index: 1, created_at: Utc::now(), updated_at: Utc::now() },
            ],
            translations: vec![],
            craftsmanship_images: vec![],
            created_at: setting.created_at,
            updated_at: Utc::now(),
        };

        let result = repo.update(id, &updated).await.unwrap();
        assert_eq!(result.hero_images.len(), 2);
        assert_eq!(result.hero_images[0].image_url, "http://new1.jpg");
        assert_eq!(result.hero_images[1].image_url, "http://new2.jpg");
    }

    #[sqlx::test]
    async fn test_update_not_found(pool: PgPool) {
        setup(&pool).await;
        let repo = SettingRepositoryImpl::new(pool.clone());

        let setting = make_setting();
        let result = repo.update(Uuid::new_v4(), &setting).await;
        assert!(matches!(result, Err(AppError::NotFound(_))));
    }

    #[sqlx::test]
    async fn test_delete(pool: PgPool) {
        setup(&pool).await;
        let repo = SettingRepositoryImpl::new(pool.clone());

        let setting = make_setting();
        repo.create(&setting).await.unwrap();

        repo.delete(setting.id).await.unwrap();

        let found = repo.find_first().await.unwrap();
        assert!(found.is_none());
    }

    #[sqlx::test]
    async fn test_delete_not_found(pool: PgPool) {
        setup(&pool).await;
        let repo = SettingRepositoryImpl::new(pool.clone());

        let result = repo.delete(Uuid::new_v4()).await;
        assert!(matches!(result, Err(AppError::NotFound(_))));
    }

    #[sqlx::test]
    async fn test_find_first_loads_all_related_data(pool: PgPool) {
        setup(&pool).await;
        let repo = SettingRepositoryImpl::new(pool.clone());
        let lang_id = insert_language(&pool).await;

        let id = Uuid::new_v4();
        let setting = Setting {
            id,
            email: "full@mebayu.com".to_string(),
            whatsapp_number: "+62811111".to_string(),
            hero_images: vec![HeroImage { id: Uuid::new_v4(), setting_id: id, image_url: "http://hero.jpg".to_string(), order_index: 0, created_at: Utc::now(), updated_at: Utc::now() }],
            craftsmanship_images: vec![CraftsmanshipImage { id: Uuid::new_v4(), setting_id: id, image_url: "http://craft.jpg".to_string(), order_index: 0, created_at: Utc::now(), updated_at: Utc::now() }],
            translations: vec![SettingTranslation {
                setting_id: id,
                language_id: lang_id,
                hero_title: "Title".to_string(),
                hero_description: "Desc".to_string(),
                about_title: "About".to_string(),
                about_description: "About Desc".to_string(),
                about_image_url: "http://about.jpg".to_string(),
            }],
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        repo.create(&setting).await.unwrap();

        let found = repo.find_first().await.unwrap().unwrap();
        assert_eq!(found.email, "full@mebayu.com");
        assert_eq!(found.hero_images.len(), 1);
        assert_eq!(found.craftsmanship_images.len(), 1);
        assert_eq!(found.translations.len(), 1);
        assert_eq!(found.translations[0].hero_title, "Title");
    }
}
