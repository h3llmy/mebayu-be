use axum::{
    Json, Router,
    extract::{Path, State},
    routing::get,
};
use uuid::Uuid;

use crate::{
    core::{
        error::AppError,
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
}

async fn get_all(
    State(state): State<Arc<AppState>>,
    ValidatedQuery(query): ValidatedQuery<PaginationQuery>,
) -> Result<Json<PaginationResponse<Vec<ProductCategory>>>, AppError> {
    let (categories, total_data) = state.product_category_service.get_all(&query).await?;
    let limit = query.get_limit();
    let total_page = (total_data as f64 / limit as f64).ceil() as u64;

    Ok(Json(PaginationResponse {
        data: categories,
        page: query.get_page(),
        limit,
        total_data,
        total_page,
    }))
}

async fn create(
    State(state): State<Arc<AppState>>,
    ValidatedJson(payload): ValidatedJson<CreateProductCategoryRequest>,
) -> Result<Json<ApiResponse<ProductCategory>>, AppError> {
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

async fn update(
    State(state): State<Arc<AppState>>,
    id: Path<Uuid>,
    ValidatedJson(payload): ValidatedJson<UpdateProductCategoryRequest>,
) -> Result<Json<ApiResponse<ProductCategory>>, AppError> {
    let category = state.product_category_service.update(*id, payload).await?;
    Ok(Json(ApiResponse { data: category }))
}

async fn delete(
    State(state): State<Arc<AppState>>,
    id: Path<Uuid>,
) -> Result<Json<ApiResponse<()>>, AppError> {
    state.product_category_service.delete(*id).await?;
    Ok(Json(ApiResponse { data: () }))
}
