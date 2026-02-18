use axum::{
    Json, Router,
    extract::{Path, State},
    routing::get,
};
use uuid::Uuid;

use crate::{
    core::{
        error::AppError,
        middleware::auth::AuthUser,
        validation::{ValidatedJson, ValidatedQuery},
    },
    domain::product_categories::{
        dto::{CreateProductCategoryRequest, UpdateProductCategoryRequest},
        entity::ProductCategory,
    },
    shared::{
        app_state::AppState,
        dto::{
            pagination::PaginationQuery,
            response::{ApiResponse, PaginationResponse},
        },
    },
};

use std::sync::Arc;
pub fn category_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/", get(get_all).post(create))
        .route("/{id}", get(get_by_id).put(update).delete(delete))
        .route("/with-product-count", get(get_all_with_product_count))
}

#[utoipa::path(
    get,
    path = "/api/v1/product-categories",
    params(
        PaginationQuery
    ),
    responses(
        (status = 200, description = "List all product categories", body = PaginationResponse<Vec<ProductCategory>>),
    )
)]
pub async fn get_all(
    State(state): State<Arc<AppState>>,
    ValidatedQuery(query): ValidatedQuery<PaginationQuery>,
) -> Result<Json<PaginationResponse<Vec<ProductCategory>>>, AppError> {
    let response = state.product_category_service.get_all(&query).await?;
    Ok(Json(response))
}

#[utoipa::path(
    post,
    path = "/api/v1/product-categories",
    request_body = CreateProductCategoryRequest,
    responses(
        (status = 201, description = "Product category created successfully", body = ApiResponse<ProductCategory>),
        (status = 400, description = "Bad Request"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden")
    ),
    security(
        ("jwt" = [])
    )
)]
pub async fn create(
    // auth_user: AuthUser,
    State(state): State<Arc<AppState>>,
    ValidatedJson(payload): ValidatedJson<CreateProductCategoryRequest>,
) -> Result<Json<ApiResponse<ProductCategory>>, AppError> {
    // auth_user.require_admin()?;
    let category = state.product_category_service.create(payload).await?;
    Ok(Json(ApiResponse { data: category }))
}

#[utoipa::path(
    get,
    path = "/api/v1/product-categories/{id}",
    responses(
        (status = 200, description = "Get product category by ID", body = ApiResponse<ProductCategory>),
        (status = 404, description = "Product category not found")
    ),
    params(
        ("id" = Uuid, Path, description = "Product Category ID")
    )
)]
pub async fn get_by_id(
    State(state): State<Arc<AppState>>,
    id: Path<Uuid>,
) -> Result<Json<ApiResponse<ProductCategory>>, AppError> {
    let category = state.product_category_service.get_by_id(*id).await?;
    Ok(Json(ApiResponse { data: category }))
}

#[utoipa::path(
    get,
    path = "/api/v1/product-categories/with-product-count",
    params(
        PaginationQuery
    ),
    responses(
        (status = 200, description = "List all product categories with product count", body = PaginationResponse<Vec<ProductCategory>>),
    )
)]
pub async fn get_all_with_product_count(
    State(state): State<Arc<AppState>>,
    ValidatedQuery(query): ValidatedQuery<PaginationQuery>,
) -> Result<Json<PaginationResponse<Vec<ProductCategory>>>, AppError> {
    let response = state
        .product_category_service
        .get_all_with_product_count(&query)
        .await?;
    Ok(Json(response))
}

#[utoipa::path(
    put,
    path = "/api/v1/product-categories/{id}",
    request_body = UpdateProductCategoryRequest,
    responses(
        (status = 200, description = "Product category updated successfully", body = ApiResponse<ProductCategory>),
        (status = 400, description = "Bad Request"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Product category not found")
    ),
    params(
        ("id" = Uuid, Path, description = "Product Category ID")
    ),
    security(
        ("jwt" = [])
    )
)]
pub async fn update(
    auth_user: AuthUser,
    State(state): State<Arc<AppState>>,
    id: Path<Uuid>,
    ValidatedJson(payload): ValidatedJson<UpdateProductCategoryRequest>,
) -> Result<Json<ApiResponse<ProductCategory>>, AppError> {
    auth_user.require_admin()?;
    let category = state.product_category_service.update(*id, payload).await?;
    Ok(Json(ApiResponse { data: category }))
}

#[utoipa::path(
    delete,
    path = "/api/v1/product-categories/{id}",
    responses(
        (status = 200, description = "Product category deleted successfully"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Product category not found")
    ),
    params(
        ("id" = Uuid, Path, description = "Product Category ID")
    ),
    security(
        ("jwt" = [])
    )
)]
pub async fn delete(
    auth_user: AuthUser,
    State(state): State<Arc<AppState>>,
    id: Path<Uuid>,
) -> Result<Json<ApiResponse<()>>, AppError> {
    auth_user.require_admin()?;
    state.product_category_service.delete(*id).await?;
    Ok(Json(ApiResponse { data: () }))
}
