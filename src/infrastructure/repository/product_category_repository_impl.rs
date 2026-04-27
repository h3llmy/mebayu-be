use crate::domain::languages::entity::Language;
use async_trait::async_trait;

use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    core::error::AppError,
    domain::product_categories::{
        entity::{ProductCategory, ProductCategoryTranslation},
        service::ProductCategoryRepository,
    },
    shared::dto::pagination::PaginationQuery,
};

pub struct ProductCategoryRepositoryImpl {
    pool: PgPool,
}

impl ProductCategoryRepositoryImpl {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl ProductCategoryRepository for ProductCategoryRepositoryImpl {
    async fn find_all(
        &self,
        query: &PaginationQuery,
        language_id: Option<Uuid>,
    ) -> Result<(Vec<ProductCategory>, u64), AppError> {
        let limit = query.get_limit() as i64;
        let offset = query.get_offset();
        let search = query.get_search().map(|s| format!("%{}%", s));

        let languages = sqlx::query_as!(
            Language,
            "SELECT id, code, name, is_default, created_at, updated_at FROM languages"
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

        let rows = sqlx::query!(
            r#"
            SELECT pc.id, pc.created_at, pc.updated_at, COUNT(*) OVER() as total_count
            FROM product_categories pc
            WHERE ($1::text IS NULL OR EXISTS (
                SELECT 1 FROM product_category_translations pct 
                WHERE pct.category_id = pc.id AND pct.name ILIKE $1
            ))
            ORDER BY pc.created_at DESC
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
        
        let mut categories = Vec::new();
        for row in rows {
            let translation_rows = sqlx::query!(
                "SELECT category_id, language_id, name FROM product_category_translations 
                 WHERE category_id = $1 AND ($2::uuid IS NULL OR language_id = $2)",
                row.id,
                language_id
            )
            .fetch_all(&self.pool)
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

            let translations: Vec<ProductCategoryTranslation> = translation_rows.into_iter().map(|t_row| {
                ProductCategoryTranslation {
                    category_id: t_row.category_id,
                    language_id: t_row.language_id,
                    language: languages.iter().find(|l| l.id == t_row.language_id).cloned(),
                    name: t_row.name,
                }
            }).collect();

            categories.push(ProductCategory {
                id: row.id,
                created_at: row.created_at,
                updated_at: row.updated_at,
                translations,
            });
        }

        Ok((categories, total as u64))
    }

    async fn find_all_with_product_count(
        &self,
        query: &PaginationQuery,
        language_id: Option<Uuid>,
    ) -> Result<(Vec<ProductCategory>, u64), AppError> {
        // Reuse find_all for now, or implement product count logic if needed
        self.find_all(query, language_id).await
    }

    async fn find_by_id(&self, id: Uuid, language_id: Option<Uuid>) -> Result<ProductCategory, AppError> {
        let row = sqlx::query!(
            "SELECT id, created_at, updated_at FROM product_categories WHERE id = $1",
            id
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?
        .ok_or_else(|| AppError::NotFound("Product category not found".to_string()))?;

        let languages = sqlx::query_as!(
            Language,
            "SELECT id, code, name, is_default, created_at, updated_at FROM languages"
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

        let translation_rows = sqlx::query!(
            "SELECT category_id, language_id, name FROM product_category_translations 
             WHERE category_id = $1 AND ($2::uuid IS NULL OR language_id = $2)",
            id,
            language_id
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

        let translations: Vec<ProductCategoryTranslation> = translation_rows.into_iter().map(|t_row| {
            ProductCategoryTranslation {
                category_id: t_row.category_id,
                language_id: t_row.language_id,
                language: languages.iter().find(|l| l.id == t_row.language_id).cloned(),
                name: t_row.name,
            }
        }).collect();

        Ok(ProductCategory {
            id: row.id,
            created_at: row.created_at,
            updated_at: row.updated_at,
            translations,
        })
    }

    async fn create(&self, category: &ProductCategory) -> Result<ProductCategory, AppError> {
        let mut tx = self.pool.begin().await.map_err(|e| AppError::Database(e.to_string()))?;

        sqlx::query!(
            "INSERT INTO product_categories (id, created_at, updated_at) VALUES ($1, $2, $3)",
            category.id,
            category.created_at,
            category.updated_at
        )
        .execute(&mut *tx)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

        for translation in &category.translations {
            sqlx::query!(
                "INSERT INTO product_category_translations (category_id, language_id, name) VALUES ($1, $2, $3)",
                category.id,
                translation.language_id,
                translation.name
            )
            .execute(&mut *tx)
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;
        }

        tx.commit().await.map_err(|e| AppError::Database(e.to_string()))?;

        self.find_by_id(category.id, None).await
    }

    async fn update(
        &self,
        id: Uuid,
        category: &ProductCategory,
    ) -> Result<ProductCategory, AppError> {
        let mut tx = self.pool.begin().await.map_err(|e| AppError::Database(e.to_string()))?;

        sqlx::query!(
            "UPDATE product_categories SET updated_at = $2 WHERE id = $1",
            id,
            category.updated_at
        )
        .execute(&mut *tx)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

        sqlx::query!("DELETE FROM product_category_translations WHERE category_id = $1", id)
            .execute(&mut *tx)
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

        for translation in &category.translations {
            sqlx::query!(
                "INSERT INTO product_category_translations (category_id, language_id, name) VALUES ($1, $2, $3)",
                id,
                translation.language_id,
                translation.name
            )
            .execute(&mut *tx)
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;
        }

        tx.commit().await.map_err(|e| AppError::Database(e.to_string()))?;

        self.find_by_id(id, None).await
    }

    async fn delete(&self, id: Uuid) -> Result<(), AppError> {
        sqlx::query("DELETE FROM product_categories WHERE id = $1")
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
    use sqlx::PgPool;
    use uuid::Uuid;

    const LANGUAGE_ID: Uuid = Uuid::from_u128(1);

    async fn setup_db(pool: &PgPool) {
        run_migrations(pool).await;
        // Insert a test language
        sqlx::query!(
            "INSERT INTO languages (id, code, name, is_default) VALUES ($1, $2, $3, $4) ON CONFLICT DO NOTHING",
            LANGUAGE_ID,
            "en",
            "English",
            true
        )
        .execute(pool)
        .await
        .unwrap();
    }

    fn sample_category(name: &str) -> ProductCategory {
        let id = Uuid::new_v4();
        ProductCategory {
            id,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            translations: vec![ProductCategoryTranslation {
                category_id: id,
                language_id: LANGUAGE_ID,
                name: name.to_string(),
            }],
        }
    }

    #[sqlx::test]
    async fn test_create_and_find_by_id(pool: PgPool) {
        setup_db(&pool).await;
        let repo = ProductCategoryRepositoryImpl::new(pool.clone());

        let category = sample_category("Electronics");

        let created = repo.create(&category).await.unwrap();
        assert_eq!(created.translations[0].name, "Electronics");

        let found = repo.find_by_id(category.id, None).await.unwrap();
        assert_eq!(found.id, category.id);
        assert_eq!(found.translations[0].name, "Electronics");
    }

    #[sqlx::test]
    async fn test_find_by_id_not_found(pool: PgPool) {
        setup_db(&pool).await;
        let repo = ProductCategoryRepositoryImpl::new(pool.clone());

        let result = repo.find_by_id(Uuid::new_v4(), None).await;
        assert!(result.is_err());
    }

    #[sqlx::test]
    async fn test_find_all(pool: PgPool) {
        setup_db(&pool).await;
        let repo = ProductCategoryRepositoryImpl::new(pool.clone());

        for i in 0..3 {
            repo.create(&sample_category(&format!("Category {}", i)))
                .await
                .unwrap();
        }

        let query = PaginationQuery {
            page: Some(1),
            search: None,
            limit: Some(10),
            sort: None,
            sort_order: None,
        };

        let (items, total) = repo.find_all(&query, None).await.unwrap();

        assert_eq!(total, 3);
        assert_eq!(items.len(), 3);
    }

    #[sqlx::test]
    async fn test_update(pool: PgPool) {
        setup_db(&pool).await;
        let repo = ProductCategoryRepositoryImpl::new(pool.clone());

        let mut category = sample_category("Old Name");
        repo.create(&category).await.unwrap();

        category.translations[0].name = "New Name".to_string();
        category.updated_at = Utc::now();

        let updated = repo.update(category.id, &category).await.unwrap();
        assert_eq!(updated.translations[0].name, "New Name");
    }

    #[sqlx::test]
    async fn test_delete(pool: PgPool) {
        setup_db(&pool).await;
        let repo = ProductCategoryRepositoryImpl::new(pool.clone());

        let category = sample_category("DeleteMe");
        repo.create(&category).await.unwrap();

        repo.delete(category.id).await.unwrap();

        let result = repo.find_by_id(category.id, None).await;
        assert!(result.is_err());
    }
}
