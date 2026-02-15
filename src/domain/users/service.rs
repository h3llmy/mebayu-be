use async_trait::async_trait;
use chrono::Utc;
use std::sync::Arc;
use uuid::Uuid;

use crate::{
    core::error::AppError,
    domain::users::dto::{CreateUserDto, UpdateUserDto},
    shared::dto::pagination::PaginationQuery,
};

use super::entity::{User, UserRole};

#[async_trait]
pub trait UserRepository: Send + Sync {
    async fn find_all(&self, query: &PaginationQuery) -> Result<(Vec<User>, u64), AppError>;
    async fn find_by_id(&self, id: Uuid) -> Result<User, AppError>;
    async fn find_by_username(&self, username: &str) -> Result<User, AppError>;
    async fn find_by_email(&self, email: &str) -> Result<User, AppError>;
    async fn create(&self, user: &User) -> Result<User, AppError>;
    async fn update(&self, id: Uuid, user: &User) -> Result<User, AppError>;
    async fn delete(&self, id: Uuid) -> Result<(), AppError>;
}

pub struct UserServiceImpl<R: UserRepository> {
    repository: Arc<R>,
}

impl<R: UserRepository> UserServiceImpl<R> {
    pub fn new(repository: Arc<R>) -> Self {
        Self { repository }
    }

    pub async fn get_all(&self, query: &PaginationQuery) -> Result<(Vec<User>, u64), AppError> {
        self.repository.find_all(query).await
    }

    pub async fn get_by_id(&self, id: Uuid) -> Result<User, AppError> {
        self.repository.find_by_id(id).await
    }

    pub async fn get_by_username(&self, username: &str) -> Result<User, AppError> {
        self.repository.find_by_username(username).await
    }

    pub async fn get_by_email(&self, email: &str) -> Result<User, AppError> {
        self.repository.find_by_email(email).await
    }

    pub async fn create(&self, req: CreateUserDto) -> Result<User, AppError> {
        let password_hash = crate::core::security::password::hash_password(&req.password)?;

        let user = User {
            id: Uuid::new_v4(),
            username: req.username,
            email: req.email,
            password_hash,
            role: UserRole::User,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        self.repository.create(&user).await
    }

    pub async fn update(&self, id: Uuid, req: UpdateUserDto) -> Result<User, AppError> {
        let user = self.repository.find_by_id(id).await?;

        let password_hash = match req.password {
            Some(p) => crate::core::security::password::hash_password(&p)?,
            None => user.password_hash,
        };

        let updated_user = User {
            id,
            username: req.username.unwrap_or(user.username),
            email: req.email.unwrap_or(user.email),
            password_hash,
            role: user.role,
            created_at: user.created_at,
            updated_at: Utc::now(),
        };
        self.repository.update(id, &updated_user).await
    }

    pub async fn delete(&self, id: Uuid) -> Result<(), AppError> {
        self.repository.delete(id).await
    }
}
