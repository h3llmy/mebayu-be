use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    core::error::AppError,
    domain::product_foundations::{entity::ProductFoundation, service::ProductFoundationRepository},
    shared::dto::pagination::{PaginationQuery, SortOrder},
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

        let allowed_sort_fields = ["name", "created_at", "updated_at"];

        let sort_field = query
            .get_sort()
            .filter(|field| allowed_sort_fields.contains(&field.as_str()))
            .unwrap_or_else(|| "created_at".to_string());

        let sort_order = match query.get_sort_order() {
            Some(SortOrder::Asc) => "ASC",
            _ => "DESC",
        };

        #[derive(sqlx::FromRow)]
        struct ProductFoundationWithCount {
            #[sqlx(flatten)]
            foundation: ProductFoundation,
            total_count: i64,
        }

        let rows = sqlx::query_as::<_, ProductFoundationWithCount>(&format!(
            r#"
            SELECT *, COUNT(*) OVER() as total_count
            FROM product_foundations
            {}
            ORDER BY {} {}
            LIMIT $1 OFFSET $2
            "#,
            if search.is_some() {
                "WHERE name ILIKE $3"
            } else {
                ""
            },
            sort_field,
            sort_order
        ))
        .bind(limit)
        .bind(offset)
        .bind(search)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

        let total = rows.first().map(|r| r.total_count).unwrap_or(0);
        let foundations = rows.into_iter().map(|r| r.foundation).collect();

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

#[cfg(test)]
mod tests {
    use crate::infrastructure::database::migrations::run_migrations;

    use super::*;
    use chrono::Utc;
    use sqlx::PgPool;
    use uuid::Uuid;

    fn sample_foundation(name: &str) -> ProductFoundation {
        ProductFoundation {
            id: Uuid::new_v4(),
            name: name.to_string(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    #[sqlx::test]
    async fn test_create_and_find_by_id(pool: PgPool) {
        run_migrations(&pool).await;
        let repo = ProductFoundationRepositoryImpl::new(pool.clone());
        let foundation = sample_foundation("Foundation A");

        let created = repo.create(&foundation).await.unwrap();
        assert_eq!(created.name, "Foundation A");

        let found = repo.find_by_id(foundation.id).await.unwrap();
        assert_eq!(found.id, foundation.id);
        assert_eq!(found.name, "Foundation A");
    }

    #[sqlx::test]
    async fn test_find_all_sort_by_name_asc(pool: PgPool) {
        run_migrations(&pool).await;
        let repo = ProductFoundationRepositoryImpl::new(pool.clone());
        
        repo.create(&sample_foundation("C")).await.unwrap();
        repo.create(&sample_foundation("A")).await.unwrap();
        repo.create(&sample_foundation("B")).await.unwrap();

        let query = PaginationQuery {
            sort: Some("name".to_string()),
            sort_order: Some(SortOrder::Asc),
            ..Default::default()
        };

        let (items, total) = repo.find_all(&query).await.unwrap();
        assert_eq!(total, 3);
        assert_eq!(items[0].name, "A");
        assert_eq!(items[1].name, "B");
        assert_eq!(items[2].name, "C");
    }
    
    #[sqlx::test]
    async fn test_find_all_sort_by_name_desc(pool: PgPool) {
        run_migrations(&pool).await;
        let repo = ProductFoundationRepositoryImpl::new(pool.clone());
        
        repo.create(&sample_foundation("C")).await.unwrap();
        repo.create(&sample_foundation("A")).await.unwrap();
        repo.create(&sample_foundation("B")).await.unwrap();

        let query = PaginationQuery {
            sort: Some("name".to_string()),
            sort_order: Some(SortOrder::Desc),
            ..Default::default()
        };

        let (items, total) = repo.find_all(&query).await.unwrap();
        assert_eq!(total, 3);
        assert_eq!(items[0].name, "C");
        assert_eq!(items[1].name, "B");
        assert_eq!(items[2].name, "A");
    }
}
