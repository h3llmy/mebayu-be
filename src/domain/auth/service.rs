use crate::{
    core::{
        error::AppError,
        security::{jwt, password},
    },
    domain::{
        auth::dto::{AuthResponseDto, LoginDto, RefreshTokenDto, RegisterDto},
        users::{
            dto::{CreateUserDto, UserResponseDto},
            entity::UserRole,
            service::UserServiceImpl,
        },
    },
};
use std::{str::FromStr, sync::Arc};
use uuid::Uuid;

pub struct AuthService {
    user_service: Arc<UserServiceImpl>,
    jwt_secret: String,
}

impl AuthService {
    pub fn new(user_service: Arc<UserServiceImpl>, jwt_secret: String) -> Self {
        Self {
            user_service,
            jwt_secret,
        }
    }

    pub async fn login(&self, req: LoginDto) -> Result<AuthResponseDto, AppError> {
        let username = req.username;
        let password_str = req.password;

        let user = self
            .user_service
            .get_by_username(&username)
            .await
            .map_err(|_| AppError::Unauthorized("Invalid username or password".to_string()))?;

        if !password::verify_password(&password_str, &user.password_hash)? {
            return Err(AppError::Unauthorized(
                "Invalid username or password".to_string(),
            ));
        }

        let tokens = jwt::generate_token_pair(
            user.id,
            UserRole::from_str(&user.role).unwrap(),
            &self.jwt_secret,
        )?;

        Ok(AuthResponseDto {
            user: UserResponseDto::from(user),
            access_token: tokens.access_token,
            refresh_token: tokens.refresh_token,
        })
    }

    pub async fn register(&self, req: RegisterDto) -> Result<AuthResponseDto, AppError> {
        let username = req.username;
        let email = req.email;
        let password_str = req.password;

        let create_user_dto = CreateUserDto {
            username,
            email,
            password: password_str,
        };

        let created_user = self.user_service.create(create_user_dto).await?;
        let tokens = jwt::generate_token_pair(
            created_user.id,
            UserRole::from_str(&created_user.role).unwrap(),
            &self.jwt_secret,
        )?;

        Ok(AuthResponseDto {
            user: UserResponseDto::from(created_user),
            access_token: tokens.access_token,
            refresh_token: tokens.refresh_token,
        })
    }

    pub async fn refresh_token(&self, req: RefreshTokenDto) -> Result<AuthResponseDto, AppError> {
        let refresh_token = req.refresh_token;

        let claims = jwt::verify_token(&refresh_token, &self.jwt_secret)?;

        if claims.token_type != jwt::TokenType::Refresh {
            return Err(AppError::Unauthorized("Invalid token type".to_string()));
        }

        let user = self.user_service.get_by_id(claims.sub).await?;
        let tokens = jwt::generate_token_pair(
            user.id,
            UserRole::from_str(&user.role).unwrap(),
            &self.jwt_secret,
        )?;

        Ok(AuthResponseDto {
            user: UserResponseDto::from(user),
            access_token: tokens.access_token,
            refresh_token: tokens.refresh_token,
        })
    }

    pub async fn get_profile(&self, user_id: Uuid) -> Result<UserResponseDto, AppError> {
        let user = self.user_service.get_by_id(user_id).await?;
        Ok(UserResponseDto::from(user))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        core::{config::Config, security::password},
        domain::users::{
            entity::User,
            service::{MockUserRepository, UserServiceImpl},
        },
    };
    use chrono::Utc;
    use mockall::predicate::*;
    use std::sync::Arc;

    fn build_user_service(mock_repo: MockUserRepository) -> Arc<UserServiceImpl> {
        let config = Config::default();
        Arc::new(UserServiceImpl::new(Arc::new(mock_repo), config))
    }

    fn sample_user_with_password(password: &str) -> User {
        User {
            id: Uuid::new_v4(),
            username: "testuser".to_string(),
            email: "test@example.com".to_string(),
            password_hash: password::hash_password(password).unwrap(),
            role: UserRole::User.to_string(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    #[tokio::test]
    async fn test_login_success() {
        let mut mock_repo = MockUserRepository::new();
        let user = sample_user_with_password("password123");

        let user_clone = user.clone();
        mock_repo
            .expect_find_by_username()
            .with(eq("testuser"))
            .returning(move |_| Ok(user_clone.clone()));

        let user_service = build_user_service(mock_repo);
        let auth_service = AuthService::new(user_service, "secret".to_string());

        let result = auth_service
            .login(LoginDto {
                username: "testuser".to_string(),
                password: "password123".to_string(),
            })
            .await
            .unwrap();

        assert_eq!(result.user.username, "testuser");
        assert!(!result.access_token.is_empty());
        assert!(!result.refresh_token.is_empty());
    }

    #[tokio::test]
    async fn test_login_wrong_password() {
        let mut mock_repo = MockUserRepository::new();
        let user = sample_user_with_password("correct");

        mock_repo
            .expect_find_by_username()
            .returning(move |_| Ok(user.clone()));

        let user_service = build_user_service(mock_repo);
        let auth_service = AuthService::new(user_service, "secret".to_string());

        let result = auth_service
            .login(LoginDto {
                username: "testuser".to_string(),
                password: "wrong".to_string(),
            })
            .await;

        assert!(matches!(result, Err(AppError::Unauthorized(_))));
    }

    #[tokio::test]
    async fn test_register_success() {
        let mut mock_repo = MockUserRepository::new();

        mock_repo.expect_create().returning(|user| Ok(user.clone()));

        let user_service = build_user_service(mock_repo);
        let auth_service = AuthService::new(user_service, "secret".to_string());

        let result = auth_service
            .register(RegisterDto {
                username: "newuser".to_string(),
                email: "new@example.com".to_string(),
                password: "password".to_string(),
            })
            .await
            .unwrap();

        assert_eq!(result.user.username, "newuser");
        assert!(!result.access_token.is_empty());
        assert!(!result.refresh_token.is_empty());
    }

    #[tokio::test]
    async fn test_refresh_token_success() {
        let mut mock_repo = MockUserRepository::new();
        let user = sample_user_with_password("password");

        let user_clone = user.clone();
        mock_repo
            .expect_find_by_id()
            .with(eq(user.id))
            .returning(move |_| Ok(user_clone.clone()));

        let user_service = build_user_service(mock_repo);
        let auth_service = AuthService::new(user_service.clone(), "secret".to_string());

        // Generate real refresh token
        let tokens = jwt::generate_token_pair(user.id, UserRole::User, "secret").unwrap();

        let result = auth_service
            .refresh_token(RefreshTokenDto {
                refresh_token: tokens.refresh_token,
            })
            .await
            .unwrap();

        assert_eq!(result.user.id, user.id);
        assert!(!result.access_token.is_empty());
    }

    #[tokio::test]
    async fn test_refresh_token_invalid_type() {
        let mock_repo = MockUserRepository::new();
        let user_service = build_user_service(mock_repo);
        let auth_service = AuthService::new(user_service, "secret".to_string());

        // Generate ACCESS token instead of refresh
        let tokens = jwt::generate_token_pair(Uuid::new_v4(), UserRole::User, "secret").unwrap();

        let result = auth_service
            .refresh_token(RefreshTokenDto {
                refresh_token: tokens.access_token,
            })
            .await;

        assert!(matches!(result, Err(AppError::Unauthorized(_))));
    }

    #[tokio::test]
    async fn test_get_profile() {
        let mut mock_repo = MockUserRepository::new();
        let user = sample_user_with_password("password");

        let user_clone = user.clone();
        mock_repo
            .expect_find_by_id()
            .with(eq(user.id))
            .returning(move |_| Ok(user_clone.clone()));

        let user_service = build_user_service(mock_repo);
        let auth_service = AuthService::new(user_service, "secret".to_string());

        let result = auth_service.get_profile(user.id).await.unwrap();

        assert_eq!(result.username, "testuser");
    }
}
