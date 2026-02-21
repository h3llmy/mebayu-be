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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::infrastructure::database::migrations::run_migrations;
    use chrono::Utc;
    use sqlx::{Pool, Postgres};
    use uuid::Uuid;

    async fn setup_db(pool: &Pool<Postgres>) {
        run_migrations(pool).await;
    }

    fn sample_user(role: UserRole) -> User {
        User {
            id: Uuid::new_v4(),
            username: format!("user_{}", Uuid::new_v4()),
            email: format!("user_{}@test.com", Uuid::new_v4()),
            password_hash: "hashed_password".to_string(),
            role: role.to_string(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    #[sqlx::test]
    async fn test_create_and_find_by_id(pool: Pool<Postgres>) {
        setup_db(&pool).await;
        let repo = UserRepositoryImpl::new(pool.clone());

        let user = sample_user(UserRole::User);

        let created = repo.create(&user).await.unwrap();
        assert_eq!(created.username, user.username);

        let found = repo.find_by_id(user.id).await.unwrap();
        assert_eq!(found.id, user.id);
        assert_eq!(found.email, user.email);
    }

    #[sqlx::test]
    async fn test_find_by_id_not_found(pool: Pool<Postgres>) {
        setup_db(&pool).await;
        let repo = UserRepositoryImpl::new(pool.clone());

        let result = repo.find_by_id(Uuid::new_v4()).await;
        assert!(matches!(result, Err(AppError::NotFound(_))));
    }

    #[sqlx::test]
    async fn test_find_by_username(pool: Pool<Postgres>) {
        setup_db(&pool).await;
        let repo = UserRepositoryImpl::new(pool.clone());

        let user = sample_user(UserRole::User);
        repo.create(&user).await.unwrap();

        let found = repo.find_by_username(&user.username).await.unwrap();
        assert_eq!(found.id, user.id);
    }

    #[sqlx::test]
    async fn test_find_all(pool: Pool<Postgres>) {
        setup_db(&pool).await;
        let repo = UserRepositoryImpl::new(pool.clone());

        for _ in 0..3 {
            repo.create(&sample_user(UserRole::User)).await.unwrap();
        }

        let query = PaginationQuery {
            page: Some(1),
            limit: Some(10),
            search: None,
            sort: None,
            sort_order: None,
        };

        let (users, total) = repo.find_all(&query).await.unwrap();

        assert_eq!(total, 3);
        assert_eq!(users.len(), 3);
    }

    #[sqlx::test]
    async fn test_find_all_pagination(pool: Pool<Postgres>) {
        setup_db(&pool).await;
        let repo = UserRepositoryImpl::new(pool.clone());

        for _ in 0..5 {
            repo.create(&sample_user(UserRole::User)).await.unwrap();
        }

        let query = PaginationQuery {
            page: Some(2),
            limit: Some(2),
            search: None,
            sort: None,
            sort_order: None,
        };

        let (users, total) = repo.find_all(&query).await.unwrap();

        assert_eq!(total, 5);
        assert_eq!(users.len(), 2);
    }

    #[sqlx::test]
    async fn test_is_admin_exists(pool: Pool<Postgres>) {
        setup_db(&pool).await;
        let repo = UserRepositoryImpl::new(pool.clone());

        // Initially no admin
        let exists = repo.is_admin_exists().await.unwrap();
        assert!(!exists);

        // Insert admin
        repo.create(&sample_user(UserRole::Admin)).await.unwrap();

        let exists_after = repo.is_admin_exists().await.unwrap();
        assert!(exists_after);
    }

    #[sqlx::test]
    async fn test_update(pool: Pool<Postgres>) {
        setup_db(&pool).await;
        let repo = UserRepositoryImpl::new(pool.clone());

        let mut user = sample_user(UserRole::User);
        repo.create(&user).await.unwrap();

        user.username = "updated_username".to_string();
        user.updated_at = Utc::now();

        let updated = repo.update(user.id, &user).await.unwrap();
        assert_eq!(updated.username, "updated_username");
    }

    #[sqlx::test]
    async fn test_delete(pool: Pool<Postgres>) {
        setup_db(&pool).await;
        let repo = UserRepositoryImpl::new(pool.clone());

        let user = sample_user(UserRole::User);
        repo.create(&user).await.unwrap();

        repo.delete(user.id).await.unwrap();

        let result = repo.find_by_id(user.id).await;
        assert!(matches!(result, Err(AppError::NotFound(_))));
    }
}
