use axum::{
    Json, Router,
    extract::{Path, State},
    routing::get,
};
use uuid::Uuid;
use std::sync::Arc;

use crate::{
    core::{
        error::{AppError, ErrorResponse},
        middleware::auth::AuthUser,
        validation::{ValidatedJson, ValidatedQuery},
    },
    domain::{
        product_foundations::{
            dto::{CreateProductFoundationRequest, UpdateProductFoundationRequest},
            entity::ProductFoundation,
        },
        users::entity::UserRole,
    },
    shared::{
        app_state::AppState,
        dto::{
            pagination::PaginationQuery,
            response::{ApiResponse, PaginationResponse},
        },
    },
};

pub fn foundation_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/", get(get_all).post(create))
        .route("/{id}", get(get_by_id).put(update).delete(delete))
}

#[utoipa::path(
    get,
    operation_id = "list_foundations",
    path = "/api/v1/product-foundations",
    params(PaginationQuery),
    responses(
        (status = 200, description = "List all foundations", body = PaginationResponse<Vec<ProductFoundation>>),
    )
)]
pub async fn get_all(
    State(state): State<Arc<AppState>>,
    ValidatedQuery(query): ValidatedQuery<PaginationQuery>,
) -> Result<Json<PaginationResponse<Vec<ProductFoundation>>>, AppError> {
    let response = state.product_foundation_service.get_all(&query).await?;
    Ok(Json(response))
}

#[utoipa::path(
    post,
    operation_id = "create_foundation",
    path = "/api/v1/product-foundations",
    request_body = CreateProductFoundationRequest,
    responses(
        (status = 201, description = "Foundation created successfully", body = ApiResponse<ProductFoundation>),
        (status = 400, description = "Bad Request", body = ErrorResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 403, description = "Forbidden", body = ErrorResponse)
    ),
    security(("jwt" = []))
)]
pub async fn create(
    auth_user: AuthUser,
    State(state): State<Arc<AppState>>,
    ValidatedJson(req): ValidatedJson<CreateProductFoundationRequest>,
) -> Result<Json<ApiResponse<ProductFoundation>>, AppError> {
    auth_user.require_role(&[UserRole::Admin])?;
    let foundation = state.product_foundation_service.create(req).await?;
    Ok(Json(ApiResponse { data: foundation }))
}

#[utoipa::path(
    get,
    operation_id = "get_foundation_by_id",
    path = "/api/v1/product-foundations/{id}",
    responses(
        (status = 200, description = "Get foundation by ID", body = ApiResponse<ProductFoundation>),
        (status = 404, description = "Foundation not found", body = ErrorResponse)
    ),
    params(("id" = Uuid, Path, description = "Foundation ID"))
)]
pub async fn get_by_id(
    State(state): State<Arc<AppState>>,
    id: Path<Uuid>,
) -> Result<Json<ApiResponse<ProductFoundation>>, AppError> {
    let foundation = state.product_foundation_service.get_by_id(*id).await?;
    Ok(Json(ApiResponse { data: foundation }))
}

#[utoipa::path(
    put,
    operation_id = "update_foundation",
    path = "/api/v1/product-foundations/{id}",
    request_body = UpdateProductFoundationRequest,
    responses(
        (status = 200, description = "Foundation updated successfully", body = ApiResponse<ProductFoundation>),
        (status = 400, description = "Bad Request", body = ErrorResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 403, description = "Forbidden", body = ErrorResponse),
        (status = 404, description = "Foundation not found", body = ErrorResponse)
    ),
    params(("id" = Uuid, Path, description = "Foundation ID")),
    security(("jwt" = []))
)]
pub async fn update(
    auth_user: AuthUser,
    State(state): State<Arc<AppState>>,
    id: Path<Uuid>,
    ValidatedJson(payload): ValidatedJson<UpdateProductFoundationRequest>,
) -> Result<Json<ApiResponse<ProductFoundation>>, AppError> {
    auth_user.require_role(&[UserRole::Admin])?;
    let foundation = state.product_foundation_service.update(*id, payload).await?;
    Ok(Json(ApiResponse { data: foundation }))
}

#[utoipa::path(
    delete,
    operation_id = "delete_foundation",
    path = "/api/v1/product-foundations/{id}",
    responses(
        (status = 200, description = "Foundation deleted successfully"),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 403, description = "Forbidden", body = ErrorResponse),
        (status = 404, description = "Foundation not found", body = ErrorResponse)
    ),
    params(("id" = Uuid, Path, description = "Foundation ID")),
    security(("jwt" = []))
)]
pub async fn delete(
    auth_user: AuthUser,
    State(state): State<Arc<AppState>>,
    id: Path<Uuid>,
) -> Result<Json<ApiResponse<()>>, AppError> {
    auth_user.require_role(&[UserRole::Admin])?;
    state.product_foundation_service.delete(*id).await?;
    Ok(Json(ApiResponse { data: () }))
}
