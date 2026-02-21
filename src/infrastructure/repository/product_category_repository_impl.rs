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

    fn sample_category(name: &str) -> ProductCategory {
        ProductCategory {
            id: Uuid::new_v4(),
            name: name.to_string(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    #[sqlx::test]
    async fn test_create_and_find_by_id(pool: PgPool) {
        setup_db(&pool).await;
        let repo = ProductCategoryRepositoryImpl::new(pool.clone());

        let category = sample_category("Electronics");

        let created = repo.create(&category).await.unwrap();
        assert_eq!(created.name, "Electronics");

        let found = repo.find_by_id(category.id).await.unwrap();
        assert_eq!(found.id, category.id);
        assert_eq!(found.name, "Electronics");
    }

    #[sqlx::test]
    async fn test_find_by_id_not_found(pool: PgPool) {
        setup_db(&pool).await;
        let repo = ProductCategoryRepositoryImpl::new(pool.clone());

        let result = repo.find_by_id(Uuid::new_v4()).await;
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

        let (items, total) = repo.find_all(&query).await.unwrap();

        assert_eq!(total, 3);
        assert_eq!(items.len(), 3);
    }

    #[sqlx::test]
    async fn test_find_all_pagination(pool: PgPool) {
        setup_db(&pool).await;
        let repo = ProductCategoryRepositoryImpl::new(pool.clone());

        for i in 0..5 {
            repo.create(&sample_category(&format!("Category {}", i)))
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
    async fn test_find_all_with_product_count(pool: PgPool) {
        setup_db(&pool).await;
        let repo = ProductCategoryRepositoryImpl::new(pool.clone());

        for i in 0..4 {
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

        let (items, total) = repo.find_all_with_product_count(&query).await.unwrap();

        assert_eq!(total, 4);
        assert_eq!(items.len(), 4);
    }

    #[sqlx::test]
    async fn test_update(pool: PgPool) {
        setup_db(&pool).await;
        let repo = ProductCategoryRepositoryImpl::new(pool.clone());

        let mut category = sample_category("Old Name");
        repo.create(&category).await.unwrap();

        category.name = "New Name".to_string();
        category.updated_at = Utc::now();

        let updated = repo.update(category.id, &category).await.unwrap();
        assert_eq!(updated.name, "New Name");
    }

    #[sqlx::test]
    async fn test_update_not_found(pool: PgPool) {
        setup_db(&pool).await;
        let repo = ProductCategoryRepositoryImpl::new(pool.clone());

        let category = sample_category("Does Not Exist");

        let result = repo.update(Uuid::new_v4(), &category).await;
        assert!(result.is_err());
    }

    #[sqlx::test]
    async fn test_delete(pool: PgPool) {
        setup_db(&pool).await;
        let repo = ProductCategoryRepositoryImpl::new(pool.clone());

        let category = sample_category("DeleteMe");
        repo.create(&category).await.unwrap();

        repo.delete(category.id).await.unwrap();

        let result = repo.find_by_id(category.id).await;
        assert!(result.is_err());
    }

    #[sqlx::test]
    async fn test_delete_non_existing(pool: PgPool) {
        setup_db(&pool).await;
        let repo = ProductCategoryRepositoryImpl::new(pool.clone());

        let result = repo.delete(Uuid::new_v4()).await;
        assert!(result.is_ok());
        // DELETE doesn't error if row doesn't exist in Postgres
    }
}
