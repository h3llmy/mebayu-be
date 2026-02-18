use axum::{
    Json, Router,
    extract::{Path, State},
    routing::get,
};
use std::sync::Arc;
use uuid::Uuid;

use crate::{
    core::{
        error::{AppError, ErrorResponse},
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

#[utoipa::path(
    get,
    path = "/api/v1/product-materials",
    params(
        PaginationQuery
    ),
    responses(
        (status = 200, description = "List all product materials", body = PaginationResponse<Vec<ProductMaterial>>),
    )
)]
pub async fn get_all(
    State(state): State<Arc<AppState>>,
    ValidatedQuery(query): ValidatedQuery<PaginationQuery>,
) -> Result<Json<PaginationResponse<Vec<ProductMaterial>>>, AppError> {
    let response = state.product_material_service.get_all(&query).await?;
    Ok(Json(response))
}

#[utoipa::path(
    post,
    path = "/api/v1/product-materials",
    request_body = CreateProductMaterialRequest,
    responses(
        (status = 201, description = "Product material created successfully", body = ApiResponse<ProductMaterial>),
        (status = 400, description = "Bad Request", body = ErrorResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 403, description = "Forbidden", body = ErrorResponse)
    ),
    security(
        ("jwt" = [])
    )
)]
pub async fn create(
    // auth_user: AuthUser,
    State(state): State<Arc<AppState>>,
    ValidatedJson(payload): ValidatedJson<CreateProductMaterialRequest>,
) -> Result<Json<ApiResponse<ProductMaterial>>, AppError> {
    // auth_user.require_admin()?;
    let material = state.product_material_service.create(payload).await?;
    Ok(Json(ApiResponse { data: material }))
}

#[utoipa::path(
    get,
    path = "/api/v1/product-materials/{id}",
    responses(
        (status = 200, description = "Get product material by ID", body = ApiResponse<ProductMaterial>),
        (status = 404, description = "Product material not found", body = ErrorResponse)
    ),
    params(
        ("id" = Uuid, Path, description = "Product Material ID")
    )
)]
pub async fn get_by_id(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<ProductMaterial>>, AppError> {
    let material = state.product_material_service.get_by_id(id).await?;
    Ok(Json(ApiResponse { data: material }))
}

#[utoipa::path(
    put,
    path = "/api/v1/product-materials/{id}",
    request_body = UpdateProductMaterialRequest,
    responses(
        (status = 200, description = "Product material updated successfully", body = ApiResponse<ProductMaterial>),
        (status = 400, description = "Bad Request", body = ErrorResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 403, description = "Forbidden", body = ErrorResponse),
        (status = 404, description = "Product material not found", body = ErrorResponse)
    ),
    params(
        ("id" = Uuid, Path, description = "Product Material ID")
    ),
    security(
        ("jwt" = [])
    )
)]
pub async fn update(
    auth_user: AuthUser,
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    ValidatedJson(payload): ValidatedJson<UpdateProductMaterialRequest>,
) -> Result<Json<ApiResponse<ProductMaterial>>, AppError> {
    auth_user.require_admin()?;
    let material = state.product_material_service.update(id, payload).await?;
    Ok(Json(ApiResponse { data: material }))
}

#[utoipa::path(
    delete,
    path = "/api/v1/product-materials/{id}",
    responses(
        (status = 200, description = "Product material deleted successfully"),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 403, description = "Forbidden", body = ErrorResponse),
        (status = 404, description = "Product material not found", body = ErrorResponse)
    ),
    params(
        ("id" = Uuid, Path, description = "Product Material ID")
    ),
    security(
        ("jwt" = [])
    )
)]
pub async fn delete(
    auth_user: AuthUser,
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<()>>, AppError> {
    auth_user.require_admin()?;
    state.product_material_service.delete(id).await?;
    Ok(Json(ApiResponse { data: () }))
}
