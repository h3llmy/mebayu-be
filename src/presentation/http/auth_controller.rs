use crate::{
    core::error::AppError,
    core::validation::ValidatedJson,
    domain::auth::dto::{AuthResponseDto, LoginDto, RefreshTokenDto, RegisterDto},
    shared::app_state::AppState,
};
use axum::{Json, Router, extract::State, routing::post};
use std::sync::Arc;

pub fn auth_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/login", post(login))
        .route("/register", post(register))
        .route("/refresh", post(refresh_token))
}

async fn login(
    State(state): State<Arc<AppState>>,
    ValidatedJson(req): ValidatedJson<LoginDto>,
) -> Result<Json<AuthResponseDto>, AppError> {
    let res = state.auth_service.login(req).await?;
    Ok(Json(res))
}

async fn register(
    State(state): State<Arc<AppState>>,
    ValidatedJson(req): ValidatedJson<RegisterDto>,
) -> Result<Json<AuthResponseDto>, AppError> {
    let res = state.auth_service.register(req).await?;
    Ok(Json(res))
}

async fn refresh_token(
    State(state): State<Arc<AppState>>,
    ValidatedJson(req): ValidatedJson<RefreshTokenDto>,
) -> Result<Json<AuthResponseDto>, AppError> {
    let res = state.auth_service.refresh_token(req).await?;
    Ok(Json(res))
}
