use axum::{
    Json, Router,
    extract::{State, Path},
    routing::get,
};
use std::sync::Arc;
use uuid::Uuid;

use crate::{
    core::{
        error::{AppError, ErrorResponse},
        middleware::auth::AuthUser,
        validation::ValidatedJson,
    },
    domain::{
        languages::{
            dto::{CreateLanguageRequest, UpdateLanguageRequest},
            entity::Language,
        },
        users::entity::UserRole,
    },
    shared::{
        app_state::AppState,
        dto::response::ApiResponse,
    },
};

pub fn language_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/", get(get_all).post(create))
        .route("/{id}", get(get_by_id).put(update).delete(delete))
}

#[utoipa::path(
    get,
    operation_id = "get_all_languages",
    path = "/api/v1/languages",
    responses(
        (status = 200, description = "Get all languages", body = ApiResponse<Vec<Language>>),
    )
)]
pub async fn get_all(
    State(state): State<Arc<AppState>>,
) -> Result<Json<ApiResponse<Vec<Language>>>, AppError> {
    let languages = state.language_service.get_all().await?;
    Ok(Json(ApiResponse { data: languages }))
}

#[utoipa::path(
    get,
    operation_id = "get_language_by_id",
    path = "/api/v1/languages/{id}",
    params(
        ("id" = Uuid, Path, description = "Language ID")
    ),
    responses(
        (status = 200, description = "Get language by ID", body = ApiResponse<Language>),
        (status = 404, description = "Language not found", body = ErrorResponse),
    )
)]
pub async fn get_by_id(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<Language>>, AppError> {
    let language = state.language_service.get_by_id(id).await?;
    Ok(Json(ApiResponse { data: language }))
}

#[utoipa::path(
    post,
    operation_id = "create_language",
    path = "/api/v1/languages",
    request_body = CreateLanguageRequest,
    responses(
        (status = 201, description = "Language created successfully", body = ApiResponse<Language>),
        (status = 400, description = "Bad Request", body = ErrorResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 403, description = "Forbidden", body = ErrorResponse),
    ),
    security(("jwt" = []))
)]
pub async fn create(
    auth_user: AuthUser,
    State(state): State<Arc<AppState>>,
    ValidatedJson(payload): ValidatedJson<CreateLanguageRequest>,
) -> Result<Json<ApiResponse<Language>>, AppError> {
    auth_user.require_role(&[UserRole::Admin])?;
    let language = state.language_service.create(payload).await?;
    Ok(Json(ApiResponse { data: language }))
}

#[utoipa::path(
    put,
    operation_id = "update_language",
    path = "/api/v1/languages/{id}",
    params(
        ("id" = Uuid, Path, description = "Language ID")
    ),
    request_body = UpdateLanguageRequest,
    responses(
        (status = 200, description = "Language updated successfully", body = ApiResponse<Language>),
        (status = 400, description = "Bad Request", body = ErrorResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 403, description = "Forbidden", body = ErrorResponse),
        (status = 404, description = "Language not found", body = ErrorResponse),
    ),
    security(("jwt" = []))
)]
pub async fn update(
    auth_user: AuthUser,
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    ValidatedJson(payload): ValidatedJson<UpdateLanguageRequest>,
) -> Result<Json<ApiResponse<Language>>, AppError> {
    auth_user.require_role(&[UserRole::Admin])?;
    let language = state.language_service.update(id, payload).await?;
    Ok(Json(ApiResponse { data: language }))
}

#[utoipa::path(
    delete,
    operation_id = "delete_language",
    path = "/api/v1/languages/{id}",
    params(
        ("id" = Uuid, Path, description = "Language ID")
    ),
    responses(
        (status = 200, description = "Language deleted successfully"),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 403, description = "Forbidden", body = ErrorResponse),
        (status = 404, description = "Language not found", body = ErrorResponse)
    ),
    security(("jwt" = []))
)]
pub async fn delete(
    auth_user: AuthUser,
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<()>>, AppError> {
    auth_user.require_role(&[UserRole::Admin])?;
    state.language_service.delete(id).await?;
    Ok(Json(ApiResponse { data: () }))
}
