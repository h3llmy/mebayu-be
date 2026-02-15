use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    core::error::AppError,
    domain::product_categories::{entity::ProductCategory, service::ProductCategoryRepository},
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
    ) -> Result<(Vec<ProductCategory>, u64), AppError> {
        let limit = query.get_limit() as i64;
        let offset = query.get_offset();

        #[derive(sqlx::FromRow)]
        struct ProductCategoryWithCount {
            #[sqlx(flatten)]
            category: ProductCategory,
            total_count: i64,
        }

        let rows = sqlx::query_as::<_, ProductCategoryWithCount>(
            r#"
            SELECT *, COUNT(*) OVER() as total_count
            FROM product_categories
            ORDER BY created_at DESC
            LIMIT $1 OFFSET $2
            "#,
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

        let total = rows.first().map(|r| r.total_count).unwrap_or(0);
        let categories = rows.into_iter().map(|r| r.category).collect();

        Ok((categories, total as u64))
    }

    async fn find_all_with_product_count(
        &self,
        query: &PaginationQuery,
    ) -> Result<(Vec<ProductCategory>, u64), AppError> {
        let limit = query.get_limit() as i64;
        let offset = query.get_offset();

        #[derive(sqlx::FromRow)]
        struct ProductCategoryWithCount {
            #[sqlx(flatten)]
            category: ProductCategory,
            total_count: i64,
        }

        let rows = sqlx::query_as::<_, ProductCategoryWithCount>(
            r#"
            SELECT *, COUNT(*) OVER() as total_count
            FROM product_categories
            ORDER BY created_at DESC
            LIMIT $1 OFFSET $2
            "#,
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

        let total = rows.first().map(|r| r.total_count).unwrap_or(0);
        let categories = rows.into_iter().map(|r| r.category).collect();

        Ok((categories, total as u64))
    }

    async fn find_by_id(&self, id: Uuid) -> Result<ProductCategory, AppError> {
        sqlx::query_as::<_, ProductCategory>("SELECT * FROM product_categories WHERE id = $1")
            .bind(id)
            .fetch_one(&self.pool)
            .await
            .map_err(|_| AppError::NotFound("Product category not found".to_string()))
    }

    async fn create(&self, category: &ProductCategory) -> Result<ProductCategory, AppError> {
        sqlx::query_as::<_, ProductCategory>(
            "INSERT INTO product_categories (id, name, created_at, updated_at)
             VALUES ($1, $2, $3, $4) RETURNING *",
        )
        .bind(category.id)
        .bind(&category.name)
        .bind(category.created_at)
        .bind(category.updated_at)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AppError::Database(e.to_string()))
    }

    async fn update(
        &self,
        id: Uuid,
        category: &ProductCategory,
    ) -> Result<ProductCategory, AppError> {
        sqlx::query_as::<_, ProductCategory>(
            "UPDATE product_categories SET name = $2, updated_at = $3 WHERE id = $1 RETURNING *",
        )
        .bind(id)
        .bind(&category.name)
        .bind(category.updated_at)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AppError::Database(e.to_string()))
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
