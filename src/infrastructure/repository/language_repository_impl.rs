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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::infrastructure::database::migrations::run_migrations;
    use chrono::Utc;
    use sqlx::PgPool;

    async fn setup(pool: &PgPool) {
        run_migrations(pool).await;
    }

    /// Generates a Language with a unique code that fits VARCHAR(10).
    /// Format: first 8 hex digits of the UUID → guaranteed unique and fits the column.
    fn sample_language(name_hint: &str) -> Language {
        let id = Uuid::new_v4();
        let code = id.simple().to_string()[..8].to_string();
        Language {
            id,
            code,
            name: format!("Language {}", name_hint),
            is_default: false,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    #[sqlx::test]
    async fn test_create_and_find_by_id(pool: PgPool) {
        setup(&pool).await;
        let repo = LanguageRepositoryImpl::new(pool.clone());

        let lang = sample_language("en");
        let created = repo.create(&lang).await.unwrap();

        assert_eq!(created.id, lang.id);
        assert_eq!(created.code, lang.code);

        let found = repo.find_by_id(lang.id).await.unwrap();
        assert_eq!(found.id, lang.id);
        assert_eq!(found.code, lang.code);
    }

    #[sqlx::test]
    async fn test_find_by_id_not_found(pool: PgPool) {
        setup(&pool).await;
        let repo = LanguageRepositoryImpl::new(pool.clone());

        let result = repo.find_by_id(Uuid::new_v4()).await;
        assert!(matches!(result, Err(AppError::NotFound(_))));
    }

    #[sqlx::test]
    async fn test_find_by_code(pool: PgPool) {
        setup(&pool).await;
        let repo = LanguageRepositoryImpl::new(pool.clone());

        let lang = sample_language("id");
        repo.create(&lang).await.unwrap();

        let found = repo.find_by_code(&lang.code).await.unwrap();
        assert_eq!(found.id, lang.id);
    }

    #[sqlx::test]
    async fn test_find_by_code_not_found(pool: PgPool) {
        setup(&pool).await;
        let repo = LanguageRepositoryImpl::new(pool.clone());

        let result = repo.find_by_code("notexist").await;
        assert!(matches!(result, Err(AppError::NotFound(_))));
    }

    #[sqlx::test]
    async fn test_find_all(pool: PgPool) {
        setup(&pool).await;
        let repo = LanguageRepositoryImpl::new(pool.clone());

        let initial = repo.find_all().await.unwrap().len();

        repo.create(&sample_language("a")).await.unwrap();
        repo.create(&sample_language("b")).await.unwrap();
        repo.create(&sample_language("c")).await.unwrap();

        let all = repo.find_all().await.unwrap();
        assert_eq!(all.len(), initial + 3);
    }

    #[sqlx::test]
    async fn test_update(pool: PgPool) {
        setup(&pool).await;
        let repo = LanguageRepositoryImpl::new(pool.clone());

        let lang = sample_language("en");
        repo.create(&lang).await.unwrap();

        let updated_lang = Language {
            id: lang.id,
            code: lang.code.clone(),
            name: "English (Updated)".to_string(),
            is_default: true,
            updated_at: Utc::now(),
            ..lang.clone()
        };

        let result = repo.update(lang.id, &updated_lang).await.unwrap();
        assert_eq!(result.name, "English (Updated)");
        assert!(result.is_default);
    }

    #[sqlx::test]
    async fn test_delete(pool: PgPool) {
        setup(&pool).await;
        let repo = LanguageRepositoryImpl::new(pool.clone());

        let lang = sample_language("de");
        repo.create(&lang).await.unwrap();

        repo.delete(lang.id).await.unwrap();

        let result = repo.find_by_id(lang.id).await;
        assert!(matches!(result, Err(AppError::NotFound(_))));
    }

    #[sqlx::test]
    async fn test_delete_not_found(pool: PgPool) {
        setup(&pool).await;
        let repo = LanguageRepositoryImpl::new(pool.clone());

        let result = repo.delete(Uuid::new_v4()).await;
        assert!(matches!(result, Err(AppError::NotFound(_))));
    }
}

