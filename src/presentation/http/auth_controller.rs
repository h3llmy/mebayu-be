use crate::{
    core::{
        error::{AppError, ErrorResponse},
        middleware::auth::AuthUser,
        validation::ValidatedJson,
    },
    domain::{
        auth::dto::{AuthResponseDto, LoginDto, RefreshTokenDto, RegisterDto},
        users::dto::user_response_dto::UserResponseDto,
    },
    shared::app_state::AppState,
};
use axum::{
    Json, Router,
    extract::State,
    routing::{get, post},
};
use std::sync::Arc;

pub fn auth_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/login", post(login))
        .route("/register", post(register))
        .route("/refresh", post(refresh_token))
        .route("/profile", get(get_profile))
}

#[utoipa::path(
    get,
    path = "/api/v1/auth/profile",
    responses(
        (status = 200, description = "Get current user profile", body = UserResponseDto),
        (status = 401, description = "Unauthorized", body = ErrorResponse)
    ),
    security(
        ("jwt" = [])
    )
)]
pub async fn get_profile(
    auth_user: AuthUser,
    State(state): State<Arc<AppState>>,
) -> Result<Json<crate::domain::users::dto::UserResponseDto>, AppError> {
    let res = state.auth_service.get_profile(auth_user.user_id).await?;
    Ok(Json(res))
}

#[utoipa::path(
    post,
    path = "/api/v1/auth/login",
    request_body = LoginDto,
    responses(
        (status = 200, description = "Login successful", body = AuthResponseDto),
        (status = 401, description = "Invalid credentials", body = ErrorResponse)
    )
)]
pub async fn login(
    State(state): State<Arc<AppState>>,
    ValidatedJson(req): ValidatedJson<LoginDto>,
) -> Result<Json<AuthResponseDto>, AppError> {
    let res = state.auth_service.login(req).await?;
    Ok(Json(res))
}

#[utoipa::path(
    post,
    path = "/api/v1/auth/register",
    request_body = RegisterDto,
    responses(
        (status = 201, description = "User registered successfully", body = AuthResponseDto),
        (status = 400, description = "Bad request", body = ErrorResponse)
    )
)]
pub async fn register(
    State(state): State<Arc<AppState>>,
    ValidatedJson(req): ValidatedJson<RegisterDto>,
) -> Result<Json<AuthResponseDto>, AppError> {
    let res = state.auth_service.register(req).await?;
    Ok(Json(res))
}

#[utoipa::path(
    post,
    path = "/api/v1/auth/refresh",
    request_body = RefreshTokenDto,
    responses(
        (status = 200, description = "Token refreshed successfully", body = AuthResponseDto),
        (status = 401, description = "Invalid refresh token", body = ErrorResponse)
    )
)]
pub async fn refresh_token(
    State(state): State<Arc<AppState>>,
    ValidatedJson(req): ValidatedJson<RefreshTokenDto>,
) -> Result<Json<AuthResponseDto>, AppError> {
    let res = state.auth_service.refresh_token(req).await?;
    Ok(Json(res))
}
