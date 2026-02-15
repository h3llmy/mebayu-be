use crate::{core::error::AppError, shared::app_state::AppState};
use axum::{
    extract::{FromRef, FromRequestParts},
    http::request::Parts,
};
use axum_extra::{
    TypedHeader,
    headers::{Authorization, authorization::Bearer},
};
use std::sync::Arc;

use crate::domain::users::entity::UserRole;

pub struct AuthUser {
    pub user_id: uuid::Uuid,
    pub role: UserRole,
}

impl AuthUser {
    pub fn require_admin(&self) -> Result<(), AppError> {
        if self.role == UserRole::Admin {
            Ok(())
        } else {
            Err(AppError::Forbidden("Admin access required".to_string()))
        }
    }
}

impl<S> FromRequestParts<S> for AuthUser
where
    Arc<AppState>: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let app_state = Arc::from_ref(state);

        // Get the Authorization header
        let bearer = TypedHeader::<Authorization<Bearer>>::from_request_parts(parts, state)
            .await
            .map_err(|_| AppError::Unauthorized("Invalid authorization header".to_string()))?;

        // Verify the token
        let claims = crate::core::security::jwt::verify_token(
            bearer.0.token(),
            &app_state.config.jwt_secret,
        )?;

        if claims.token_type != crate::core::security::jwt::TokenType::Access {
            return Err(AppError::Unauthorized("Invalid token type".to_string()));
        }

        Ok(AuthUser {
            user_id: claims.sub,
            role: claims.role,
        })
    }
}
