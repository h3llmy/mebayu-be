use std::{sync::Arc, time::Duration};

use axum::{Json, Router, extract::State, routing::post};

use crate::{
    core::{
        error::{AppError, ErrorResponse},
        middleware::auth::AuthUser,
        validation::ValidatedJson,
    },
    domain::users::entity::UserRole,
    infrastructure::object_storage::s3::Storage,
    shared::{
        app_state::AppState,
        dto::{
            object_storage::{GetUploadUrlRequest, GetUploadUrlResponse},
            response::ApiResponse,
        },
    },
};

pub fn storage_routes() -> Router<Arc<AppState>> {
    Router::new().route("/get-presign-url", post(get_presign_url))
}

#[utoipa::path(
    post,
    operation_id = "get_presign_url",
    path = "/api/v1/storages/get-presign-url",
    request_body = GetUploadUrlRequest,
    responses(
        (status = 200, description = "Get presigned URL for upload", body = ApiResponse<Vec<GetUploadUrlResponse>>),
        (status = 401, description = "Unauthorized", body = ErrorResponse)
    ),
    security(
        ("jwt" = [])
    )
)]
pub async fn get_presign_url(
    auth_user: AuthUser,
    State(state): State<Arc<AppState>>,
    ValidatedJson(query): ValidatedJson<GetUploadUrlRequest>,
) -> Result<Json<ApiResponse<Vec<GetUploadUrlResponse>>>, AppError> {
    auth_user.require_role(&[UserRole::Admin])?;

    let result = state
        .s3_service
        .generate_upload_url(query, Duration::from_secs(3600))
        .await?;

    Ok(Json(ApiResponse { data: result }))
}
