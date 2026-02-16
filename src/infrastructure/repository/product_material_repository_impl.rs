use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    core::error::AppError,
    domain::product_materials::{entity::ProductMaterial, service::ProductMaterialRepository},
    shared::dto::pagination::PaginationQuery,
};

pub struct ProductMaterialRepositoryImpl {
    pool: PgPool,
}

impl ProductMaterialRepositoryImpl {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl ProductMaterialRepository for ProductMaterialRepositoryImpl {
    async fn find_all(
        &self,
        query: &PaginationQuery,
    ) -> Result<(Vec<ProductMaterial>, u64), AppError> {
        let limit = query.get_limit() as i64;
        let offset = query.get_offset();

        #[derive(sqlx::FromRow)]
        struct ProductMaterialWithCount {
            #[sqlx(flatten)]
            material: ProductMaterial,
            total_count: i64,
        }

        let rows = sqlx::query_as::<_, ProductMaterialWithCount>(
            r#"
            SELECT *, COUNT(*) OVER() as total_count
            FROM product_materials
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
        let materials = rows.into_iter().map(|r| r.material).collect();

        Ok((materials, total as u64))
    }

    async fn find_by_id(&self, id: Uuid) -> Result<ProductMaterial, AppError> {
        sqlx::query_as::<_, ProductMaterial>("SELECT * FROM product_materials WHERE id = $1")
            .bind(id)
            .fetch_one(&self.pool)
            .await
            .map_err(|_| AppError::NotFound("Product material not found".to_string()))
    }

    async fn create(&self, material: &ProductMaterial) -> Result<ProductMaterial, AppError> {
        sqlx::query_as::<_, ProductMaterial>(
            "INSERT INTO product_materials (id, name, created_at, updated_at)
             VALUES ($1, $2, $3, $4) RETURNING *",
        )
        .bind(material.id)
        .bind(&material.name)
        .bind(material.created_at)
        .bind(material.updated_at)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AppError::Database(e.to_string()))
    }

    async fn update(
        &self,
        id: Uuid,
        material: &ProductMaterial,
    ) -> Result<ProductMaterial, AppError> {
        sqlx::query_as::<_, ProductMaterial>(
            "UPDATE product_materials SET name = $2, updated_at = $3 WHERE id = $1 RETURNING *",
        )
        .bind(id)
        .bind(&material.name)
        .bind(material.updated_at)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AppError::Database(e.to_string()))
    }

    async fn delete(&self, id: Uuid) -> Result<(), AppError> {
        sqlx::query("DELETE FROM product_materials WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(())
    }
}
