use async_trait::async_trait;
use chrono::Utc;
use std::sync::Arc;
use uuid::Uuid;

use crate::{
    core::{config::Config, error::AppError, security::password},
    domain::users::dto::{CreateUserDto, UpdateUserDto, UserResponseDto},
    shared::dto::{pagination::PaginationQuery, response::PaginationResponse},
};

use super::entity::{User, UserRole};

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait UserRepository: Send + Sync {
    async fn find_all(&self, query: &PaginationQuery) -> Result<(Vec<User>, u64), AppError>;
    async fn find_by_id(&self, id: Uuid) -> Result<User, AppError>;
    async fn find_by_username(&self, username: &str) -> Result<User, AppError>;
    async fn is_admin_exists(&self) -> Result<bool, AppError>;
    // async fn find_by_email(&self, email: &str) -> Result<User, AppError>;
    async fn create(&self, user: &User) -> Result<User, AppError>;
    async fn update(&self, id: Uuid, user: &User) -> Result<User, AppError>;
    async fn delete(&self, id: Uuid) -> Result<(), AppError>;
}

pub struct UserServiceImpl {
    repository: Arc<dyn UserRepository>,
    config: Config,
}

impl UserServiceImpl {
    pub fn new(repository: Arc<dyn UserRepository>, config: Config) -> Self {
        Self { repository, config }
    }

    pub async fn get_all(
        &self,
        query: &PaginationQuery,
    ) -> Result<PaginationResponse<Vec<UserResponseDto>>, AppError> {
        let (users, total_data) = self.repository.find_all(query).await?;
        let limit = query.get_limit();
        let total_page = (total_data as f64 / limit as f64).ceil() as u64;

        let data = users.into_iter().map(UserResponseDto::from).collect();

        Ok(PaginationResponse {
            data,
            page: query.get_page(),
            limit,
            total_data,
            total_page,
        })
    }

    pub async fn get_by_id(&self, id: Uuid) -> Result<User, AppError> {
        self.repository.find_by_id(id).await
    }

    pub async fn get_by_username(&self, username: &str) -> Result<User, AppError> {
        self.repository.find_by_username(username).await
    }

    // pub async fn get_by_email(&self, email: &str) -> Result<User, AppError> {
    //     self.repository.find_by_email(email).await
    // }

    pub async fn create_initial_user(&self) {
        match self.repository.is_admin_exists().await {
            Ok(exists) => {
                if exists {
                    tracing::info!("Skiping admin creation, admin already exists");
                    return;
                }
            }
            Err(error) => {
                tracing::error!("failed to check admin exists {:?}", error);
                return;
            }
        }

        let password_hash = match password::hash_password(&self.config.admin_password) {
            Ok(hash) => hash,
            Err(error) => {
                tracing::error!("failed to hash password {:?}", error);
                return;
            }
        };

        let user = User {
            id: Uuid::new_v4(),
            username: self.config.admin_username.clone(),
            email: self.config.admin_email.clone(),
            password_hash,
            role: UserRole::Admin.to_string(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        match self.repository.create(&user).await {
            Ok(_) => {
                tracing::info!("Admin created successfully");
            }
            Err(error) => {
                tracing::error!("failed to init admin {:?}", error);
            }
        };
    }

    pub async fn create(&self, req: CreateUserDto) -> Result<User, AppError> {
        let password_hash = password::hash_password(&req.password)?;

        let user = User {
            id: Uuid::new_v4(),
            username: req.username,
            email: req.email,
            password_hash,
            role: UserRole::User.to_string(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        self.repository.create(&user).await
    }

    pub async fn update(&self, id: Uuid, req: UpdateUserDto) -> Result<User, AppError> {
        let user = self.repository.find_by_id(id).await?;

        let password_hash = match req.password {
            Some(p) => password::hash_password(&p)?,
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

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    #[tokio::test]
    async fn test_get_by_id() {
        let mut mock_repo = MockUserRepository::new();
        let id = Uuid::new_v4();
        let expected_user = User {
            id,
            username: "testuser".to_string(),
            email: "test@example.com".to_string(),
            password_hash: "hash".to_string(),
            role: "user".to_string(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let user_clone = expected_user.clone();
        mock_repo
            .expect_find_by_id()
            .with(mockall::predicate::eq(id))
            .times(1)
            .returning(move |_| Ok(user_clone.clone()));

        let config = Config::default();
        let service = UserServiceImpl::new(Arc::new(mock_repo), config);
        let result = service.get_by_id(id).await.unwrap();

        assert_eq!(result.id, expected_user.id);
        assert_eq!(result.username, expected_user.username);
    }

    #[tokio::test]
    async fn test_get_by_username() {
        let mut mock_repo = MockUserRepository::new();
        let username = "testuser";
        let expected_user = User {
            id: Uuid::new_v4(),
            username: username.to_string(),
            email: "test@example.com".to_string(),
            password_hash: "hash".to_string(),
            role: "user".to_string(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let user_clone = expected_user.clone();
        mock_repo
            .expect_find_by_username()
            .with(mockall::predicate::eq(username))
            .times(1)
            .returning(move |_| Ok(user_clone.clone()));

        let config = Config::default();
        let service = UserServiceImpl::new(Arc::new(mock_repo), config);
        let result = service.get_by_username(username).await.unwrap();

        assert_eq!(result.username, expected_user.username);
    }

    #[tokio::test]
    async fn test_get_all() {
        let mut mock_repo = MockUserRepository::new();
        let query = PaginationQuery::default();
        let total_data = 1;
        let users = vec![User {
            id: Uuid::new_v4(),
            username: "test".to_string(),
            email: "test@example.com".to_string(),
            password_hash: "hash".to_string(),
            role: "user".to_string(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }];

        let users_clone = users.clone();
        mock_repo
            .expect_find_all()
            .times(1)
            .returning(move |_| Ok((users_clone.clone(), total_data)));

        let config = Config::default();
        let service = UserServiceImpl::new(Arc::new(mock_repo), config);
        let result = service.get_all(&query).await.unwrap();

        assert_eq!(result.total_data, total_data);
        assert_eq!(result.data.len(), 1);
    }

    #[tokio::test]
    async fn test_create() {
        let mut mock_repo = MockUserRepository::new();
        let req = CreateUserDto {
            username: "newuser".to_string(),
            email: "new@example.com".to_string(),
            password: "password".to_string(),
        };

        mock_repo
            .expect_create()
            .times(1)
            .returning(|user| Ok(user.clone()));

        let config = Config::default();
        let service = UserServiceImpl::new(Arc::new(mock_repo), config);
        let result = service.create(req).await.unwrap();

        assert_eq!(result.username, "newuser");
    }

    #[tokio::test]
    async fn test_update() {
        let mut mock_repo = MockUserRepository::new();
        let id = Uuid::new_v4();
        let existing = User {
            id,
            username: "olduser".to_string(),
            email: "old@example.com".to_string(),
            password_hash: "hash".to_string(),
            role: "user".to_string(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        let req = UpdateUserDto {
            username: Some("newuser".to_string()),
            email: Some("new@example.com".to_string()),
            password: None,
        };

        let existing_clone = existing.clone();
        mock_repo
            .expect_find_by_id()
            .with(mockall::predicate::eq(id))
            .returning(move |_| Ok(existing_clone.clone()));

        mock_repo
            .expect_update()
            .with(mockall::predicate::eq(id), mockall::predicate::always())
            .times(1)
            .returning(|_, updated| Ok(updated.clone()));

        let config = Config::default();
        let service = UserServiceImpl::new(Arc::new(mock_repo), config);
        let result = service.update(id, req).await.unwrap();

        assert_eq!(result.username, "newuser");
    }

    #[tokio::test]
    async fn test_delete() {
        let mut mock_repo = MockUserRepository::new();
        let id = Uuid::new_v4();

        mock_repo
            .expect_delete()
            .with(mockall::predicate::eq(id))
            .times(1)
            .returning(|_| Ok(()));

        let config = Config::default();
        let service = UserServiceImpl::new(Arc::new(mock_repo), config);
        let result = service.delete(id).await;

        assert!(result.is_ok());
    }
}
