use async_trait::async_trait;

use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    core::error::AppError,
    domain::product_foundations::{
        entity::{ProductFoundation, ProductFoundationTranslation},
        service::ProductFoundationRepository,
    },
    shared::dto::pagination::PaginationQuery,
};

pub struct ProductFoundationRepositoryImpl {
    pool: PgPool,
}

impl ProductFoundationRepositoryImpl {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl ProductFoundationRepository for ProductFoundationRepositoryImpl {
    async fn find_all(
        &self,
        query: &PaginationQuery,
    ) -> Result<(Vec<ProductFoundation>, u64), AppError> {
        let limit = query.get_limit() as i64;
        let offset = query.get_offset();
        let search = query.get_search().map(|s| format!("%{}%", s));

        let rows = sqlx::query!(
            r#"
            SELECT pf.id, pf.created_at, pf.updated_at, COUNT(*) OVER() as total_count
            FROM product_foundations pf
            WHERE ($1::text IS NULL OR EXISTS (
                SELECT 1 FROM product_foundation_translations pft 
                WHERE pft.foundation_id = pf.id AND pft.name ILIKE $1
            ))
            ORDER BY pf.created_at DESC
            LIMIT $2 OFFSET $3
            "#,
            search,
            limit,
            offset
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

        let total = rows.first().map(|r| r.total_count.unwrap_or(0)).unwrap_or(0);
        
        let mut foundations = Vec::new();
        for row in rows {
            let translations = sqlx::query_as!(
                ProductFoundationTranslation,
                "SELECT foundation_id, language_id, name FROM product_foundation_translations WHERE foundation_id = $1",
                row.id
            )
            .fetch_all(&self.pool)
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

            foundations.push(ProductFoundation {
                id: row.id,
                created_at: row.created_at,
                updated_at: row.updated_at,
                translations,
            });
        }

        Ok((foundations, total as u64))
    }

    async fn find_by_id(&self, id: Uuid) -> Result<ProductFoundation, AppError> {
        let row = sqlx::query!(
            "SELECT id, created_at, updated_at FROM product_foundations WHERE id = $1",
            id
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?
        .ok_or_else(|| AppError::NotFound("Foundation not found".to_string()))?;

        let translations = sqlx::query_as!(
            ProductFoundationTranslation,
            "SELECT foundation_id, language_id, name FROM product_foundation_translations WHERE foundation_id = $1",
            id
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(ProductFoundation {
            id: row.id,
            created_at: row.created_at,
            updated_at: row.updated_at,
            translations,
        })
    }

    async fn create(&self, foundation: &ProductFoundation) -> Result<ProductFoundation, AppError> {
        let mut tx = self.pool.begin().await.map_err(|e| AppError::Database(e.to_string()))?;

        sqlx::query!(
            "INSERT INTO product_foundations (id, created_at, updated_at) VALUES ($1, $2, $3)",
            foundation.id,
            foundation.created_at,
            foundation.updated_at
        )
        .execute(&mut *tx)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

        for translation in &foundation.translations {
            sqlx::query!(
                "INSERT INTO product_foundation_translations (foundation_id, language_id, name) VALUES ($1, $2, $3)",
                foundation.id,
                translation.language_id,
                translation.name
            )
            .execute(&mut *tx)
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;
        }

        tx.commit().await.map_err(|e| AppError::Database(e.to_string()))?;

        self.find_by_id(foundation.id).await
    }

    async fn update(
        &self,
        id: Uuid,
        foundation: &ProductFoundation,
    ) -> Result<ProductFoundation, AppError> {
        let mut tx = self.pool.begin().await.map_err(|e| AppError::Database(e.to_string()))?;

        sqlx::query!(
            "UPDATE product_foundations SET updated_at = $2 WHERE id = $1",
            id,
            foundation.updated_at
        )
        .execute(&mut *tx)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

        sqlx::query!("DELETE FROM product_foundation_translations WHERE foundation_id = $1", id)
            .execute(&mut *tx)
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

        for translation in &foundation.translations {
            sqlx::query!(
                "INSERT INTO product_foundation_translations (foundation_id, language_id, name) VALUES ($1, $2, $3)",
                id,
                translation.language_id,
                translation.name
            )
            .execute(&mut *tx)
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;
        }

        tx.commit().await.map_err(|e| AppError::Database(e.to_string()))?;

        self.find_by_id(id).await
    }

    async fn delete(&self, id: Uuid) -> Result<(), AppError> {
        sqlx::query("DELETE FROM product_foundations WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::infrastructure::database::migrations::run_migrations;
    use super::*;
    use chrono::Utc;

    async fn setup_db(pool: &PgPool) -> Uuid {
        run_migrations(pool).await;
        let id = Uuid::new_v4();
        sqlx::query!(
            "INSERT INTO languages (id, code, name, is_default) VALUES ($1, $2, $3, $4) ON CONFLICT DO NOTHING",
            id, "en", "English", true
        )
        .execute(pool)
        .await
        .unwrap();
        id
    }

    fn sample_foundation(name: &str, lang_id: Uuid) -> ProductFoundation {
        let id = Uuid::new_v4();
        ProductFoundation {
            id,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            translations: vec![ProductFoundationTranslation {
                foundation_id: id,
                language_id: lang_id,
                name: name.to_string(),
            }],
        }
    }

    #[sqlx::test]
    async fn test_create_and_find_by_id(pool: PgPool) {
        let lang_id = setup_db(&pool).await;
        let repo = ProductFoundationRepositoryImpl::new(pool.clone());
        let foundation = sample_foundation("Foundation A", lang_id);

        let created = repo.create(&foundation).await.unwrap();
        assert_eq!(created.translations[0].name, "Foundation A");

        let found = repo.find_by_id(foundation.id).await.unwrap();
        assert_eq!(found.id, foundation.id);
        assert_eq!(found.translations[0].name, "Foundation A");
    }

    #[sqlx::test]
    async fn test_find_all(pool: PgPool) {
        let lang_id = setup_db(&pool).await;
        let repo = ProductFoundationRepositoryImpl::new(pool.clone());
        
        repo.create(&sample_foundation("C", lang_id)).await.unwrap();
        repo.create(&sample_foundation("A", lang_id)).await.unwrap();
        repo.create(&sample_foundation("B", lang_id)).await.unwrap();

        let query = PaginationQuery::default();

        let (items, total) = repo.find_all(&query).await.unwrap();
        assert_eq!(total, 3);
        assert_eq!(items.len(), 3);
    }
}
