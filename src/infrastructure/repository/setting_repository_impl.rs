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
        .map_err(|e| AppError::Database(e.to_string()))?;

        if let Some(mut s) = setting {
            let images = sqlx::query_as!(
                HeroImage,
                "SELECT * FROM hero_images WHERE setting_id = $1 ORDER BY order_index",
                s.id
            )
            .fetch_all(&self.pool)
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;
            s.hero_images = images;
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
            .map_err(|e| AppError::Database(e.to_string()))?;

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
        .map_err(|e| AppError::Database(e.to_string()))?;

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
            .map_err(|e| AppError::Database(e.to_string()))?;
            created_images.push(res);
        }

        tx.commit()
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;
        setting_res.hero_images = created_images;
        Ok(setting_res)
    }

    async fn update(&self, id: Uuid, setting: &Setting) -> Result<Setting, AppError> {
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

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
        .map_err(|e| AppError::Database(e.to_string()))?
        .ok_or_else(|| AppError::NotFound("Setting not found".to_string()))?;

        // Delete old images
        sqlx::query!("DELETE FROM hero_images WHERE setting_id = $1", id)
            .execute(&mut *tx)
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

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
            .map_err(|e| AppError::Database(e.to_string()))?;
            created_images.push(res);
        }

        tx.commit()
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;
        setting_res.hero_images = created_images;
        Ok(setting_res)
    }

    async fn delete(&self, id: Uuid) -> Result<(), AppError> {
        let result = sqlx::query!("DELETE FROM settings WHERE id = $1", id)
            .execute(&self.pool)
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

        if result.rows_affected() == 0 {
            return Err(AppError::NotFound("Setting not found".to_string()));
        }

        Ok(())
    }
}
