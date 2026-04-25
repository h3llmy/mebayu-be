use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    core::error::AppError,
    domain::languages::{entity::Language, repository::LanguageRepository},
};

pub struct LanguageRepositoryImpl {
    pool: PgPool,
}

impl LanguageRepositoryImpl {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl LanguageRepository for LanguageRepositoryImpl {
    async fn find_all(&self) -> Result<Vec<Language>, AppError> {
        let res = sqlx::query_as!(Language, "SELECT * FROM languages ORDER BY code ASC")
            .fetch_all(&self.pool)
            .await
            .map_err(|e: sqlx::Error| AppError::Database(e.to_string()))?;
        Ok(res)
    }

    async fn find_by_code(&self, code: &str) -> Result<Language, AppError> {
        let res = sqlx::query_as!(Language, "SELECT * FROM languages WHERE code = $1", code)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| AppError::Database(e.to_string()))?
            .ok_or_else(|| AppError::NotFound(format!("Language with code {} not found", code)))?;
        Ok(res)
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Language, AppError> {
        let res = sqlx::query_as!(Language, "SELECT * FROM languages WHERE id = $1", id)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| AppError::Database(e.to_string()))?
            .ok_or_else(|| AppError::NotFound("Language not found".to_string()))?;
        Ok(res)
    }

    async fn create(&self, language: &Language) -> Result<Language, AppError> {
        let res = sqlx::query_as!(
            Language,
            r#"
            INSERT INTO languages (id, code, name, is_default, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING *
            "#,
            language.id,
            language.code,
            language.name,
            language.is_default,
            language.created_at,
            language.updated_at
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e: sqlx::Error| AppError::Database(e.to_string()))?;
        
        Ok(res)
    }

    async fn update(&self, id: Uuid, language: &Language) -> Result<Language, AppError> {
        let res = sqlx::query_as!(
            Language,
            r#"
            UPDATE languages 
            SET code = $2, name = $3, is_default = $4, updated_at = $5
            WHERE id = $1
            RETURNING *
            "#,
            id,
            language.code,
            language.name,
            language.is_default,
            language.updated_at
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e: sqlx::Error| AppError::Database(e.to_string()))?;

        Ok(res)
    }

    async fn delete(&self, id: Uuid) -> Result<(), AppError> {
        let result = sqlx::query!("DELETE FROM languages WHERE id = $1", id)
            .execute(&self.pool)
            .await
            .map_err(|e: sqlx::Error| AppError::Database(e.to_string()))?;

        if result.rows_affected() == 0 {
            return Err(AppError::NotFound("Language not found".to_string()));
        }

        Ok(())
    }
}
