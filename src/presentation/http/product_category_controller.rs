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

async fn get_all(
    State(state): State<Arc<AppState>>,
    ValidatedQuery(query): ValidatedQuery<PaginationQuery>,
) -> Result<Json<PaginationResponse<Vec<ProductCategory>>>, AppError> {
    let response = state.product_category_service.get_all(&query).await?;
    Ok(Json(response))
}

async fn create(
    auth_user: AuthUser,
    State(state): State<Arc<AppState>>,
    ValidatedJson(payload): ValidatedJson<CreateProductCategoryRequest>,
) -> Result<Json<ApiResponse<ProductCategory>>, AppError> {
    auth_user.require_admin()?;
    let category = state.product_category_service.create(payload).await?;
    Ok(Json(ApiResponse { data: category }))
}

async fn get_by_id(
    State(state): State<Arc<AppState>>,
    id: Path<Uuid>,
) -> Result<Json<ApiResponse<ProductCategory>>, AppError> {
    let category = state.product_category_service.get_by_id(*id).await?;
    Ok(Json(ApiResponse { data: category }))
}

async fn get_all_with_product_count(
    State(state): State<Arc<AppState>>,
    ValidatedQuery(query): ValidatedQuery<PaginationQuery>,
) -> Result<Json<PaginationResponse<Vec<ProductCategory>>>, AppError> {
    let response = state
        .product_category_service
        .get_all_with_product_count(&query)
        .await?;
    Ok(Json(response))
}

async fn update(
    auth_user: AuthUser,
    State(state): State<Arc<AppState>>,
    id: Path<Uuid>,
    ValidatedJson(payload): ValidatedJson<UpdateProductCategoryRequest>,
) -> Result<Json<ApiResponse<ProductCategory>>, AppError> {
    auth_user.require_admin()?;
    let category = state.product_category_service.update(*id, payload).await?;
    Ok(Json(ApiResponse { data: category }))
}

async fn delete(
    auth_user: AuthUser,
    State(state): State<Arc<AppState>>,
    id: Path<Uuid>,
) -> Result<Json<ApiResponse<()>>, AppError> {
    auth_user.require_admin()?;
    state.product_category_service.delete(*id).await?;
    Ok(Json(ApiResponse { data: () }))
}
