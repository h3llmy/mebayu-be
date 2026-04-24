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
            .await?;
        Ok(res)
    }

    async fn find_by_code(&self, code: &str) -> Result<Language, AppError> {
        let res = sqlx::query_as!(Language, "SELECT * FROM languages WHERE code = $1", code)
            .fetch_optional(&self.pool)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("Language with code {} not found", code)))?;
        Ok(res)
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Language, AppError> {
        let res = sqlx::query_as!(Language, "SELECT * FROM languages WHERE id = $1", id)
            .fetch_optional(&self.pool)
            .await?
            .ok_or_else(|| AppError::NotFound("Language not found".to_string()))?;
        Ok(res)
    }
}
