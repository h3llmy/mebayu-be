use crate::{
    core::error::AppError,
    domain::users::{
        entity::{User, UserRole},
        service::UserRepository,
    },
    shared::dto::pagination::PaginationQuery,
};
use async_trait::async_trait;
use sqlx::{Pool, Postgres, Row};
use uuid::Uuid;

pub struct UserRepositoryImpl {
    pool: Pool<Postgres>,
}

impl UserRepositoryImpl {
    pub fn new(pool: Pool<Postgres>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl UserRepository for UserRepositoryImpl {
    async fn find_all(&self, query: &PaginationQuery) -> Result<(Vec<User>, u64), AppError> {
        let page = query.page.unwrap_or(1);
        let limit = query.limit.unwrap_or(10);
        let offset = (page - 1) * limit;

        let rows = sqlx::query(
            r#"
        SELECT 
            u.*,
            COUNT(*) OVER() AS total_count
        FROM users u
        ORDER BY u.created_at DESC
        LIMIT $1 OFFSET $2
        "#,
        )
        .bind(limit as i64)
        .bind(offset as i64)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

        if rows.is_empty() {
            return Ok((vec![], 0));
        }

        let total = rows[0].get::<i64, _>("total_count") as u64;

        let users = rows
            .into_iter()
            .map(|row| User {
                id: row.get("id"),
                username: row.get("username"),
                email: row.get("email"),
                password_hash: row.get("password_hash"),
                role: row.get("role"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
            })
            .collect();

        Ok((users, total))
    }

    async fn find_by_id(&self, id: Uuid) -> Result<User, AppError> {
        sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| AppError::Database(e.to_string()))?
            .ok_or_else(|| AppError::NotFound("User not found".to_string()))
    }

    async fn is_admin_exists(&self) -> Result<bool, AppError> {
        let row = sqlx::query(
            "SELECT EXISTS (
                SELECT 1
                FROM users
                WHERE role = $1
            )",
        )
        .bind(UserRole::Admin.to_string())
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;
        Ok(row.get::<bool, _>("exists"))
    }

    async fn find_by_username(&self, username: &str) -> Result<User, AppError> {
        sqlx::query_as::<_, User>("SELECT * FROM users WHERE username = $1")
            .bind(username)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| AppError::Database(e.to_string()))?
            .ok_or_else(|| AppError::NotFound("User not found".to_string()))
    }

    // async fn find_by_email(&self, email: &str) -> Result<User, AppError> {
    //     sqlx::query_as::<_, User>("SELECT * FROM users WHERE email = $1")
    //         .bind(email)
    //         .fetch_optional(&self.pool)
    //         .await
    //         .map_err(|e| AppError::Database(e.to_string()))?
    //         .ok_or_else(|| AppError::NotFound("User not found".to_string()))
    // }

    async fn create(&self, user: &User) -> Result<User, AppError> {
        sqlx::query_as::<_, User>(
            "INSERT INTO users (id, username, email, password_hash, role, created_at, updated_at) 
             VALUES ($1, $2, $3, $4, $5, $6, $7) 
             RETURNING *",
        )
        .bind(user.id)
        .bind(&user.username)
        .bind(&user.email)
        .bind(&user.password_hash)
        .bind(&user.role)
        .bind(user.created_at)
        .bind(user.updated_at)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AppError::Database(e.to_string()))
    }

    async fn update(&self, id: Uuid, user: &User) -> Result<User, AppError> {
        sqlx::query_as::<_, User>(
            "UPDATE users SET username = $1, email = $2, password_hash = $3, role = $4, updated_at = $5 
             WHERE id = $6 
             RETURNING *",
        )
        .bind(&user.username)
        .bind(&user.email)
        .bind(&user.password_hash)
        .bind(&user.role)
        .bind(user.updated_at)
        .bind(id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AppError::Database(e.to_string()))
    }

    async fn delete(&self, id: Uuid) -> Result<(), AppError> {
        sqlx::query("DELETE FROM users WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;
        Ok(())
    }
}
