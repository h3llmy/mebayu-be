use crate::{
    core::{
        error::{AppError, ErrorResponse},
        middleware::auth::AuthUser,
        validation::{ValidatedJson, ValidatedQuery},
    },
    domain::users::dto::{CreateUserDto, UpdateUserDto, UserResponseDto},
    shared::{
        app_state::AppState,
        dto::{pagination::PaginationQuery, response::PaginationResponse},
    },
};
use axum::{
    Json, Router,
    extract::{Path, State},
    routing::get,
};
use std::sync::Arc;
use uuid::Uuid;

pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/", get(get_all).post(create))
        .route("/{id}", get(get_by_id).put(update).delete(delete_user))
}

#[utoipa::path(
    get,
    operation_id = "list_users",
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
    operation_id = "get_user_by_id",
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
    operation_id = "create_user",
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
    operation_id = "update_user",
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
    operation_id = "delete_user",
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
