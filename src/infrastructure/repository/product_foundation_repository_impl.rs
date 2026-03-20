use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    core::error::AppError,
    domain::product_foundations::{entity::ProductFoundation, service::ProductFoundationRepository},
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
        let limit = query.get_limit();
        let offset = query.get_offset();
        let search = query.search.as_deref().unwrap_or("");

        let foundations = sqlx::query_as!(
            ProductFoundation,
            r#"
            SELECT * FROM product_foundations 
            WHERE name ILIKE $1
            ORDER BY created_at DESC
            LIMIT $2 OFFSET $3
            "#,
            format!("%{}%", search),
            limit as i64,
            offset as i64
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

        let total = sqlx::query!(
            "SELECT count(*) FROM product_foundations WHERE name ILIKE $1",
            format!("%{}%", search)
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?
        .count
        .unwrap_or(0);

        Ok((foundations, total as u64))
    }

    async fn find_by_id(&self, id: Uuid) -> Result<ProductFoundation, AppError> {
        sqlx::query_as!(
            ProductFoundation,
            "SELECT * FROM product_foundations WHERE id = $1",
            id
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?
        .ok_or_else(|| AppError::NotFound("Foundation not found".to_string()))
    }

    async fn create(&self, foundation: &ProductFoundation) -> Result<ProductFoundation, AppError> {
        sqlx::query_as!(
            ProductFoundation,
            r#"
            INSERT INTO product_foundations (id, name, created_at, updated_at)
            VALUES ($1, $2, $3, $4)
            RETURNING *
            "#,
            foundation.id,
            foundation.name,
            foundation.created_at,
            foundation.updated_at
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AppError::Database(e.to_string()))
    }

    async fn update(
        &self,
        id: Uuid,
        foundation: &ProductFoundation,
    ) -> Result<ProductFoundation, AppError> {
        sqlx::query_as!(
            ProductFoundation,
            r#"
            UPDATE product_foundations 
            SET name = $2, updated_at = $3
            WHERE id = $1
            RETURNING *
            "#,
            id,
            foundation.name,
            foundation.updated_at
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?
        .ok_or_else(|| AppError::NotFound("Foundation not found".to_string()))
    }

    async fn delete(&self, id: Uuid) -> Result<(), AppError> {
        let result = sqlx::query!("DELETE FROM product_foundations WHERE id = $1", id)
            .execute(&self.pool)
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

        if result.rows_affected() == 0 {
            return Err(AppError::NotFound("Foundation not found".to_string()));
        }

        Ok(())
    }
}
