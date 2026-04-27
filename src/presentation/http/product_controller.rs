use axum::{
    Json, Router,
    extract::{Path, Query, State},
    routing::get,
};
use serde::Deserialize;
use uuid::Uuid;

use crate::{
    core::{
        error::{AppError, ErrorResponse},
        middleware::auth::AuthUser,
        validation::{ValidatedJson, ValidatedQuery, LanguageCode},
    },
    domain::{
        products::{
            dto::{CreateProductRequest, UpdateProductRequest, GetProductsQuery},
            entity::Product,
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

use std::sync::Arc;
pub fn product_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/", get(get_all).post(create))
        .route("/{id}", get(get_by_id).put(update).delete(delete))
        .route("/{id}/recommendations", get(get_recommendations))
}

#[utoipa::path(
    get,
    operation_id = "list_products",
    path = "/api/v1/products",
    params(
        PaginationQuery,
        ("category_id" = Option<Uuid>, Query, description = "Filter by category ID"),
        ("material_id" = Option<Uuid>, Query, description = "Filter by material ID"),
        ("foundation_id" = Option<Uuid>, Query, description = "Filter by foundation ID"),
        ("Accept-Language" = Option<String>, Header, description = "Language code for filtering translations (e.g. 'en', 'id')"),
    ),
    responses(
        (status = 200, description = "List all products", body = PaginationResponse<Vec<Product>>),
    )
)]
pub async fn get_all(
    State(state): State<Arc<AppState>>,
    ValidatedQuery(query): ValidatedQuery<GetProductsQuery>,
    LanguageCode(lang): LanguageCode,
) -> Result<Json<PaginationResponse<Vec<Product>>>, AppError> {
    let language_id = if let Some(code) = lang {
        state.language_service.get_by_code(&code).await.ok().map(|l| l.id)
    } else {
        None
    };

    let response = state.product_service.get_all(&query, language_id).await?;
    
    Ok(Json(response))
}

#[utoipa::path(
    post,
    operation_id = "create_product",
    path = "/api/v1/products",
    request_body = CreateProductRequest,
    responses(
        (status = 201, description = "Product created successfully", body = ApiResponse<Product>),
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
    ValidatedJson(req): ValidatedJson<CreateProductRequest>,
) -> Result<Json<ApiResponse<Product>>, AppError> {
    // auth_user.require_admin()?;
    let product = state.product_service.create(req).await?;
    Ok(Json(ApiResponse { data: product }))
}

#[utoipa::path(
    get,
    operation_id = "get_product_by_id",
    path = "/api/v1/products/{id}",
    responses(
        (status = 200, description = "Get product by ID", body = ApiResponse<Product>),
        (status = 404, description = "Product not found", body = ErrorResponse)
    ),
    params(
        ("id" = Uuid, Path, description = "Product ID"),
        ("Accept-Language" = Option<String>, Header, description = "Language code for filtering translations (e.g. 'en', 'id')"),
    )
)]
pub async fn get_by_id(
    State(state): State<Arc<AppState>>,
    LanguageCode(lang): LanguageCode,
    id: Path<Uuid>,
) -> Result<Json<ApiResponse<Product>>, AppError> {
    let language_id = if let Some(code) = lang {
        state.language_service.get_by_code(&code).await.ok().map(|l| l.id)
    } else {
        None
    };

    let product = state.product_service.get_by_id(*id, language_id).await?;
    
    Ok(Json(ApiResponse { data: product }))
}

#[utoipa::path(
    put,
    operation_id = "update_product",
    path = "/api/v1/products/{id}",
    request_body = UpdateProductRequest,
    responses(
        (status = 200, description = "Product updated successfully", body = ApiResponse<Product>),
        (status = 400, description = "Bad Request", body = ErrorResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 403, description = "Forbidden", body = ErrorResponse),
        (status = 404, description = "Product not found", body = ErrorResponse)
    ),
    params(
        ("id" = Uuid, Path, description = "Product ID")
    ),
    security(
        ("jwt" = [])
    )
)]
pub async fn update(
    auth_user: AuthUser,
    State(state): State<Arc<AppState>>,
    id: Path<Uuid>,
    ValidatedJson(payload): ValidatedJson<UpdateProductRequest>,
) -> Result<Json<ApiResponse<Product>>, AppError> {
    auth_user.require_role(&[UserRole::Admin])?;
    let product = state.product_service.update(*id, payload).await?;
    Ok(Json(ApiResponse { data: product }))
}

#[utoipa::path(
    delete,
    operation_id = "delete_product",
    path = "/api/v1/products/{id}",
    responses(
        (status = 200, description = "Product deleted successfully"),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 403, description = "Forbidden", body = ErrorResponse),
        (status = 404, description = "Product not found", body = ErrorResponse)
    ),
    params(
        ("id" = Uuid, Path, description = "Product ID")
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
    auth_user.require_role(&[UserRole::Admin])?;
    state.product_service.delete(*id).await?;
    Ok(Json(ApiResponse { data: () }))
}

#[derive(Debug, Deserialize)]
pub struct RecommendationsQuery {
    pub limit: Option<i64>,
}

#[utoipa::path(
    get,
    operation_id = "get_product_recommendations",
    path = "/api/v1/products/{id}/recommendations",
    params(
        ("id" = Uuid, Path, description = "Product ID"),
        ("limit" = Option<i64>, Query, description = "Max number of recommendations to return (default 8, max 50)"),
        ("Accept-Language" = Option<String>, Header, description = "Language code for filtering translations (e.g. 'en', 'id')"),
    ),
    responses(
        (status = 200, description = "Product recommendations", body = ApiResponse<Vec<Product>>),
        (status = 404, description = "Product not found", body = ErrorResponse)
    )
)]
pub async fn get_recommendations(
    State(state): State<Arc<AppState>>,
    LanguageCode(lang): LanguageCode,
    id: Path<Uuid>,
    Query(query): Query<RecommendationsQuery>,
) -> Result<Json<ApiResponse<Vec<Product>>>, AppError> {
    let language_id = if let Some(code) = lang {
        state.language_service.get_by_code(&code).await.ok().map(|l| l.id)
    } else {
        None
    };

    let products = state
        .product_service
        .get_recommendations(*id, query.limit, language_id)
        .await?;
    
    Ok(Json(ApiResponse { data: products }))
}
