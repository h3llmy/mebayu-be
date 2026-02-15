use axum::{
    Json, Router,
    extract::{Path, State},
    routing::get,
};
use uuid::Uuid;

use crate::{
    core::validation::{ValidatedJson, ValidatedQuery},
    domain::users::dto::{CreateUserDto, UpdateUserDto, UserResponseDto},
    shared::{
        app_state::AppState,
        dto::{pagination::PaginationQuery, response::PaginationResponse},
    },
};

use crate::core::middleware::auth::AuthUser;
use std::sync::Arc;

pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/", get(get_all).post(create))
        .route("/{id}", get(get_by_id).put(update).delete(delete_user))
}

async fn get_all(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    ValidatedQuery(query): ValidatedQuery<PaginationQuery>,
) -> Result<Json<PaginationResponse<Vec<UserResponseDto>>>, crate::core::error::AppError> {
    auth.require_admin()?;
    let (users, total) = state.user_service.get_all(&query).await?;
    let response = users.into_iter().map(UserResponseDto::from).collect();

    let limit = query.get_limit();
    let total_page = (total as f64 / limit as f64).ceil() as u64;

    Ok(Json(PaginationResponse {
        data: response,
        total_data: total,
        page: query.get_page(),
        limit,
        total_page,
    }))
}

async fn get_by_id(
    _auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<UserResponseDto>, crate::core::error::AppError> {
    let user = state.user_service.get_by_id(id).await?;
    Ok(Json(UserResponseDto::from(user)))
}

async fn create(
    State(state): State<Arc<AppState>>,
    ValidatedJson(req): ValidatedJson<CreateUserDto>,
) -> Result<Json<UserResponseDto>, crate::core::error::AppError> {
    let user = state.user_service.create(req).await?;
    Ok(Json(UserResponseDto::from(user)))
}

async fn update(
    _auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    ValidatedJson(req): ValidatedJson<UpdateUserDto>,
) -> Result<Json<UserResponseDto>, crate::core::error::AppError> {
    let user = state.user_service.update(id, req).await?;
    Ok(Json(UserResponseDto::from(user)))
}

async fn delete_user(
    _auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<()>, crate::core::error::AppError> {
    state.user_service.delete(id).await?;
    Ok(Json(()))
}
