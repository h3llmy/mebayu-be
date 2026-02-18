use axum::{
    Json, Router,
    extract::{Path, State},
    routing::get,
};
use uuid::Uuid;

use crate::{
    core::{
        error::{AppError, ErrorResponse},
        validation::{ValidatedJson, ValidatedQuery},
    },
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

#[utoipa::path(
    get,
    path = "/api/v1/users",
    params(
        PaginationQuery
    ),
    responses(
        (status = 200, description = "List all users", body = PaginationResponse<Vec<UserResponseDto>>),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 403, description = "Forbidden", body = ErrorResponse)
    ),
    security(
        ("jwt" = [])
    )
)]
pub async fn get_all(
    // auth: AuthUser,
    State(state): State<Arc<AppState>>,
    ValidatedQuery(query): ValidatedQuery<PaginationQuery>,
) -> Result<Json<PaginationResponse<Vec<UserResponseDto>>>, AppError> {
    // auth.require_admin()?;
    let response = state.user_service.get_all(&query).await?;
    Ok(Json(response))
}

#[utoipa::path(
    get,
    path = "/api/v1/users/{id}",
    responses(
        (status = 200, description = "Get user by ID", body = UserResponseDto),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 404, description = "User not found", body = ErrorResponse)
    ),
    params(
        ("id" = Uuid, Path, description = "User ID")
    ),
    security(
        ("jwt" = [])
    )
)]
pub async fn get_by_id(
    _auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<UserResponseDto>, AppError> {
    let user = state.user_service.get_by_id(id).await?;
    Ok(Json(UserResponseDto::from(user)))
}

#[utoipa::path(
    post,
    path = "/api/v1/users",
    request_body = CreateUserDto,
    responses(
        (status = 201, description = "User created successfully", body = UserResponseDto),
        (status = 400, description = "Bad Request", body = ErrorResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 403, description = "Forbidden", body = ErrorResponse)
    ),
    security(
        ("jwt" = [])
    )
)]
pub async fn create(
    State(state): State<Arc<AppState>>,
    ValidatedJson(req): ValidatedJson<CreateUserDto>,
) -> Result<Json<UserResponseDto>, AppError> {
    let user = state.user_service.create(req).await?;
    Ok(Json(UserResponseDto::from(user)))
}

#[utoipa::path(
    put,
    path = "/api/v1/users/{id}",
    request_body = UpdateUserDto,
    responses(
        (status = 200, description = "User updated successfully", body = UserResponseDto),
        (status = 400, description = "Bad Request", body = ErrorResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 403, description = "Forbidden", body = ErrorResponse),
        (status = 404, description = "User not found", body = ErrorResponse)
    ),
    params(
        ("id" = Uuid, Path, description = "User ID")
    ),
    security(
        ("jwt" = [])
    )
)]
pub async fn update(
    _auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    ValidatedJson(req): ValidatedJson<UpdateUserDto>,
) -> Result<Json<UserResponseDto>, AppError> {
    let user = state.user_service.update(id, req).await?;
    Ok(Json(UserResponseDto::from(user)))
}

#[utoipa::path(
    delete,
    path = "/api/v1/users/{id}",
    responses(
        (status = 200, description = "User deleted successfully"),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 403, description = "Forbidden", body = ErrorResponse),
        (status = 404, description = "User not found", body = ErrorResponse)
    ),
    params(
        ("id" = Uuid, Path, description = "User ID")
    ),
    security(
        ("jwt" = [])
    )
)]
pub async fn delete_user(
    _auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<()>, AppError> {
    state.user_service.delete(id).await?;
    Ok(Json(()))
}
