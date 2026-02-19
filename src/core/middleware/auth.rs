use crate::{
    core::{error::AppError, security::jwt},
    shared::app_state::AppState,
};
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
    pub fn require_role(&self, role: &[UserRole]) -> Result<(), AppError> {
        if role.contains(&self.role) {
            Ok(())
        } else {
            Err(AppError::Forbidden(format!(
                "Access Denied {:?} role required",
                role
            )))
        }
    }

    #[allow(dead_code)]
    pub fn require_permission(&self, _permission: &str) -> Result<(), AppError> {
        unimplemented!("Permission check not implemented yet")
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

        let bearer = TypedHeader::<Authorization<Bearer>>::from_request_parts(parts, state)
            .await
            .map_err(|_| AppError::Unauthorized("Unauthorized".to_string()))?;

        let claims = jwt::verify_token(bearer.0.token(), &app_state.config.jwt_secret)?;

        if claims.token_type != jwt::TokenType::Access {
            return Err(AppError::Unauthorized("Unauthorized".to_string()));
        }

        Ok(AuthUser {
            user_id: claims.sub,
            role: claims.role,
        })
    }
}
