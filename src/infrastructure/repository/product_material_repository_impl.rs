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

#[cfg(test)]
mod tests {
    use crate::infrastructure::database::migrations::run_migrations;

    use super::*;
    use chrono::Utc;
    use sqlx::PgPool;
    use uuid::Uuid;

    async fn setup_db(pool: &PgPool) {
        run_migrations(pool).await;
    }

    fn sample_material(name: &str) -> ProductMaterial {
        ProductMaterial {
            id: Uuid::new_v4(),
            name: name.to_string(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    #[sqlx::test]
    async fn test_create_and_find_by_id(pool: PgPool) {
        setup_db(&pool).await;
        let repo = ProductMaterialRepositoryImpl::new(pool.clone());

        let material = sample_material("Steel");

        let created = repo.create(&material).await.unwrap();
        assert_eq!(created.name, "Steel");

        let found = repo.find_by_id(material.id).await.unwrap();
        assert_eq!(found.id, material.id);
        assert_eq!(found.name, "Steel");
    }

    #[sqlx::test]
    async fn test_find_by_id_not_found(pool: PgPool) {
        setup_db(&pool).await;
        let repo = ProductMaterialRepositoryImpl::new(pool.clone());

        let result = repo.find_by_id(Uuid::new_v4()).await;
        assert!(result.is_err());
    }

    #[sqlx::test]
    async fn test_find_all(pool: PgPool) {
        setup_db(&pool).await;
        let repo = ProductMaterialRepositoryImpl::new(pool.clone());

        for i in 0..3 {
            repo.create(&sample_material(&format!("Material {}", i)))
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

        let (items, total) = repo.find_all(&query).await.unwrap();

        assert_eq!(total, 3);
        assert_eq!(items.len(), 3);
    }

    #[sqlx::test]
    async fn test_find_all_pagination(pool: PgPool) {
        setup_db(&pool).await;
        let repo = ProductMaterialRepositoryImpl::new(pool.clone());

        for i in 0..5 {
            repo.create(&sample_material(&format!("Material {}", i)))
                .await
                .unwrap();
        }

        let query = PaginationQuery {
            page: Some(2),
            search: None,
            limit: Some(2),
            sort: None,
            sort_order: None,
        };

        let (items, total) = repo.find_all(&query).await.unwrap();

        assert_eq!(total, 5);
        assert_eq!(items.len(), 2);
    }

    #[sqlx::test]
    async fn test_update(pool: PgPool) {
        setup_db(&pool).await;
        let repo = ProductMaterialRepositoryImpl::new(pool.clone());

        let mut material = sample_material("Old Name");
        repo.create(&material).await.unwrap();

        material.name = "New Name".to_string();
        material.updated_at = Utc::now();

        let updated = repo.update(material.id, &material).await.unwrap();
        assert_eq!(updated.name, "New Name");
    }

    #[sqlx::test]
    async fn test_update_not_found(pool: PgPool) {
        setup_db(&pool).await;
        let repo = ProductMaterialRepositoryImpl::new(pool.clone());

        let material = sample_material("Does Not Exist");

        let result = repo.update(Uuid::new_v4(), &material).await;
        assert!(result.is_err());
    }

    #[sqlx::test]
    async fn test_delete(pool: PgPool) {
        setup_db(&pool).await;
        let repo = ProductMaterialRepositoryImpl::new(pool.clone());

        let material = sample_material("DeleteMe");
        repo.create(&material).await.unwrap();

        repo.delete(material.id).await.unwrap();

        let result = repo.find_by_id(material.id).await;
        assert!(result.is_err());
    }

    #[sqlx::test]
    async fn test_delete_non_existing(pool: PgPool) {
        setup_db(&pool).await;
        let repo = ProductMaterialRepositoryImpl::new(pool.clone());

        let result = repo.delete(Uuid::new_v4()).await;
        assert!(result.is_ok());
        // Postgres DELETE does not error if row does not exist
    }
}
