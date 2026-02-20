use std::{sync::Arc, time::Duration};

use axum::{Json, Router, extract::State, routing::get};

use crate::{
    core::{
        error::{AppError, ErrorResponse},
        middleware::auth::AuthUser,
        validation::ValidatedQuery,
    },
    domain::users::entity::UserRole,
    shared::{
        app_state::AppState,
        dto::{
            object_storage::{GetUploadUrlRequest, GetUploadUrlResponse},
            response::ApiResponse,
        },
    },
};

pub fn storage_routes() -> Router<Arc<AppState>> {
    Router::new().route("/get-presign-url", get(get_presign_url))
}

#[utoipa::path(
    get,
    operation_id = "get_presign_url",
    path = "/api/v1/storages/get-presign-url",
    params(
        GetUploadUrlRequest
    ),
    responses(
        (status = 200, description = "Get presigned URL for upload", body = ApiResponse<GetUploadUrlResponse>),
        (status = 401, description = "Unauthorized", body = ErrorResponse)
    ),
    security(
        ("jwt" = [])
    )
)]
pub async fn get_presign_url(
    auth_user: AuthUser,
    State(state): State<Arc<AppState>>,
    ValidatedQuery(query): ValidatedQuery<GetUploadUrlRequest>,
) -> Result<Json<ApiResponse<GetUploadUrlResponse>>, AppError> {
    auth_user.require_role(&[UserRole::Admin])?;

    let (upload_url, public_url, file_key) = state
        .s3_service
        .generate_upload_url(
            &query.file_name,
            &query.folder,
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
