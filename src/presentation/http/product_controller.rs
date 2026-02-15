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
    domain::products::{
        dto::{CreateProductRequest, UpdateProductRequest},
        entity::Product,
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
pub fn product_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/", get(get_all).post(create))
        .route("/{id}", get(get_by_id).put(update).delete(delete))
}

async fn get_all(
    State(state): State<Arc<AppState>>,
    ValidatedQuery(query): ValidatedQuery<PaginationQuery>,
) -> Result<Json<PaginationResponse<Vec<Product>>>, AppError> {
    let (products, total_data) = state.product_service.get_all(&query).await?;
    let limit = query.get_limit();
    let total_page = (total_data as f64 / limit as f64).ceil() as u64;

    Ok(Json(PaginationResponse {
        data: products,
        page: query.get_page(),
        limit,
        total_data,
        total_page,
    }))
}

async fn create(
    State(state): State<Arc<AppState>>,
    ValidatedJson(payload): ValidatedJson<CreateProductRequest>,
) -> Result<Json<ApiResponse<Product>>, AppError> {
    let product = state.product_service.create(payload).await?;
    Ok(Json(ApiResponse { data: product }))
}

async fn get_by_id(
    State(state): State<Arc<AppState>>,
    id: Path<Uuid>,
) -> Result<Json<ApiResponse<Product>>, AppError> {
    let product = state.product_service.get_by_id(*id).await?;
    Ok(Json(ApiResponse { data: product }))
}

async fn update(
    State(state): State<Arc<AppState>>,
    id: Path<Uuid>,
    ValidatedJson(payload): ValidatedJson<UpdateProductRequest>,
) -> Result<Json<ApiResponse<Product>>, AppError> {
    let product = state.product_service.update(*id, payload).await?;
    Ok(Json(ApiResponse { data: product }))
}

async fn delete(
    State(state): State<Arc<AppState>>,
    id: Path<Uuid>,
) -> Result<Json<ApiResponse<()>>, AppError> {
    state.product_service.delete(*id).await?;
    Ok(Json(ApiResponse { data: () }))
}
