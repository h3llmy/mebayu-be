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
            service::{UserRepository, UserServiceImpl},
        },
    },
};
use std::{str::FromStr, sync::Arc};
use uuid::Uuid;

pub struct AuthService<R: UserRepository> {
    user_service: Arc<UserServiceImpl<R>>,
    jwt_secret: String,
}

impl<R: UserRepository> AuthService<R> {
    pub fn new(user_service: Arc<UserServiceImpl<R>>, jwt_secret: String) -> Self {
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
