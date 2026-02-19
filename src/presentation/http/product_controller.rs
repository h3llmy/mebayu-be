use axum::{
    Json, Router,
    extract::{Path, State},
    routing::get,
};
use uuid::Uuid;

use crate::{
    core::{
        error::{AppError, ErrorResponse},
        middleware::auth::AuthUser,
        validation::{ValidatedJson, ValidatedQuery},
    },
    domain::{
        products::{
            dto::{
                CreateProductRequest, GetUploadUrlRequest, GetUploadUrlResponse,
                UpdateProductRequest,
            },
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
use std::time::Duration;
pub fn product_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/", get(get_all).post(create))
        .route("/upload-url", get(get_upload_url))
        .route("/{id}", get(get_by_id).put(update).delete(delete))
}

#[utoipa::path(
    get,
    path = "/api/v1/products/upload-url",
    responses(
        (status = 200, description = "Get presigned URL for upload", body = ApiResponse<GetUploadUrlResponse>),
        (status = 401, description = "Unauthorized", body = ErrorResponse)
    ),
    params(
        GetUploadUrlRequest
    ),
    security(
        ("jwt" = [])
    )
)]
pub async fn get_upload_url(
    // auth_user: AuthUser,
    State(state): State<Arc<AppState>>,
    ValidatedQuery(query): ValidatedQuery<GetUploadUrlRequest>,
) -> Result<Json<ApiResponse<GetUploadUrlResponse>>, AppError> {
    // auth_user.require_role(&[UserRole::Admin])?;

    let (upload_url, public_url, file_key) = state
        .s3_service
        .generate_upload_url(
            &query.file_name,
            "products",
            &query.content_type,
            Duration::from_secs(3600),
        )
        .await?;

    Ok(Json(ApiResponse {
        data: GetUploadUrlResponse {
            upload_url,
            public_url,
            file_key,
        },
    }))
}

#[utoipa::path(
    get,
    operation_id = "list_products",
    path = "/api/v1/products",
    params(
        PaginationQuery
    ),
    responses(
        (status = 200, description = "List all products", body = PaginationResponse<Vec<Product>>),
    )
)]
pub async fn get_all(
    State(state): State<Arc<AppState>>,
    ValidatedQuery(query): ValidatedQuery<PaginationQuery>,
) -> Result<Json<PaginationResponse<Vec<Product>>>, AppError> {
    let response = state.product_service.get_all(&query).await?;
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
        ("id" = Uuid, Path, description = "Product ID")
    )
)]
pub async fn get_by_id(
    State(state): State<Arc<AppState>>,
    id: Path<Uuid>,
) -> Result<Json<ApiResponse<Product>>, AppError> {
    let product = state.product_service.get_by_id(*id).await?;
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
