use crate::{
    core::{
        error::AppError,
        security::{jwt, password},
    },
    domain::{
        auth::dto::{AuthResponseDto, LoginDto, RefreshTokenDto, RegisterDto},
        users::{
            dto::UserResponseDto,
            entity::{User, UserRole},
            service::UserRepository,
        },
    },
};
use chrono::Utc;
use std::sync::Arc;
use uuid::Uuid;

pub struct AuthService<R: UserRepository> {
    repository: Arc<R>,
    jwt_secret: String,
}

impl<R: UserRepository> AuthService<R> {
    pub fn new(repository: Arc<R>, jwt_secret: String) -> Self {
        Self {
            repository,
            jwt_secret,
        }
    }

    pub async fn login(&self, req: LoginDto) -> Result<AuthResponseDto, AppError> {
        let username = req.username.unwrap();
        let password_str = req.password.unwrap();

        let user = self
            .repository
            .find_by_username(&username)
            .await
            .map_err(|_| AppError::Unauthorized("Invalid username or password".to_string()))?;

        if !password::verify_password(&password_str, &user.password_hash)? {
            return Err(AppError::Unauthorized(
                "Invalid username or password".to_string(),
            ));
        }

        let tokens = jwt::generate_token_pair(user.id, user.role.clone(), &self.jwt_secret)?;

        Ok(AuthResponseDto {
            user: UserResponseDto::from(user),
            access_token: tokens.access_token,
            refresh_token: tokens.refresh_token,
        })
    }

    pub async fn register(&self, req: RegisterDto) -> Result<AuthResponseDto, AppError> {
        let username = req.username.unwrap();
        let email = req.email.unwrap();
        let password_str = req.password.unwrap();

        let password_hash = password::hash_password(&password_str).unwrap();

        let user = User {
            id: Uuid::new_v4(),
            username,
            email,
            password_hash,
            role: UserRole::User,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let created_user = self.repository.create(&user).await?;
        let tokens =
            jwt::generate_token_pair(created_user.id, created_user.role.clone(), &self.jwt_secret)?;

        Ok(AuthResponseDto {
            user: UserResponseDto::from(created_user),
            access_token: tokens.access_token,
            refresh_token: tokens.refresh_token,
        })
    }

    pub async fn refresh_token(&self, req: RefreshTokenDto) -> Result<AuthResponseDto, AppError> {
        let refresh_token = req.refresh_token.unwrap();

        let claims = jwt::verify_token(&refresh_token, &self.jwt_secret)?;

        if claims.token_type != jwt::TokenType::Refresh {
            return Err(AppError::Unauthorized("Invalid token type".to_string()));
        }

        let user = self.repository.find_by_id(claims.sub).await?;
        let tokens = jwt::generate_token_pair(user.id, user.role.clone(), &self.jwt_secret)?;

        Ok(AuthResponseDto {
            user: UserResponseDto::from(user),
            access_token: tokens.access_token,
            refresh_token: tokens.refresh_token,
        })
    }
}
