use axum::{
    Json, Router,
    extract::{Path, State},
    routing::get,
};
use std::sync::Arc;
use uuid::Uuid;

use crate::{
    core::{
        error::AppError,
        middleware::auth::AuthUser,
        validation::{ValidatedJson, ValidatedQuery},
    },
    domain::product_materials::{
        dto::{CreateProductMaterialRequest, UpdateProductMaterialRequest},
        entity::ProductMaterial,
    },
    shared::{
        app_state::AppState,
        dto::{
            pagination::PaginationQuery,
            response::{ApiResponse, PaginationResponse},
        },
    },
};

pub fn product_material_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/", get(get_all).post(create))
        .route("/{id}", get(get_by_id).put(update).delete(delete))
}

async fn get_all(
    State(state): State<Arc<AppState>>,
    ValidatedQuery(query): ValidatedQuery<PaginationQuery>,
) -> Result<Json<PaginationResponse<Vec<ProductMaterial>>>, AppError> {
    let response = state.product_material_service.get_all(&query).await?;
    Ok(Json(response))
}

async fn create(
    // auth_user: AuthUser,
    State(state): State<Arc<AppState>>,
    ValidatedJson(payload): ValidatedJson<CreateProductMaterialRequest>,
) -> Result<Json<ApiResponse<ProductMaterial>>, AppError> {
    // auth_user.require_admin()?;
    let material = state.product_material_service.create(payload).await?;
    Ok(Json(ApiResponse { data: material }))
}

async fn get_by_id(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<ProductMaterial>>, AppError> {
    let material = state.product_material_service.get_by_id(id).await?;
    Ok(Json(ApiResponse { data: material }))
}

async fn update(
    auth_user: AuthUser,
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    ValidatedJson(payload): ValidatedJson<UpdateProductMaterialRequest>,
) -> Result<Json<ApiResponse<ProductMaterial>>, AppError> {
    auth_user.require_admin()?;
    let material = state.product_material_service.update(id, payload).await?;
    Ok(Json(ApiResponse { data: material }))
}

async fn delete(
    auth_user: AuthUser,
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<()>>, AppError> {
    auth_user.require_admin()?;
    state.product_material_service.delete(id).await?;
    Ok(Json(ApiResponse { data: () }))
}
