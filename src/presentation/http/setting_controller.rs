use axum::{
    Json, Router,
    extract::State,
    routing::get,
};
use std::sync::Arc;

use crate::{
    core::{
        error::{AppError, ErrorResponse},
        middleware::auth::AuthUser,
        validation::ValidatedJson,
    },
    domain::{
        settings::{
            dto::request::UpdateSettingRequest,
            entity::Setting,
        },
        users::entity::UserRole,
    },
    shared::{
        app_state::AppState,
        dto::response::ApiResponse,
    },
};

pub fn setting_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/", get(get_setting).put(update).delete(delete))
}

#[utoipa::path(
    get,
    operation_id = "get_setting",
    path = "/api/v1/settings",
    responses(
        (status = 200, description = "Get the website setting (returns default if not set)", body = ApiResponse<Setting>),
    )
)]
pub async fn get_setting(
    State(state): State<Arc<AppState>>,
) -> Result<Json<ApiResponse<Setting>>, AppError> {
    let setting = state.setting_service.get_first().await?;
    Ok(Json(ApiResponse { data: setting }))
}

#[utoipa::path(
    put,
    operation_id = "upsert_setting",
    path = "/api/v1/settings",
    request_body = UpdateSettingRequest,
    responses(
        (status = 200, description = "Setting created or updated successfully", body = ApiResponse<Setting>),
        (status = 400, description = "Bad Request", body = ErrorResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 403, description = "Forbidden", body = ErrorResponse),
    ),
    security(("jwt" = []))
)]
pub async fn update(
    auth_user: AuthUser,
    State(state): State<Arc<AppState>>,
    ValidatedJson(payload): ValidatedJson<UpdateSettingRequest>,
) -> Result<Json<ApiResponse<Setting>>, AppError> {
    auth_user.require_role(&[UserRole::Admin])?;
    let setting = state.setting_service.upsert(payload).await?;
    Ok(Json(ApiResponse { data: setting }))
}

#[utoipa::path(
    delete,
    operation_id = "delete_setting",
    path = "/api/v1/settings",
    responses(
        (status = 200, description = "Setting deleted successfully"),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 403, description = "Forbidden", body = ErrorResponse),
        (status = 404, description = "Setting not found", body = ErrorResponse)
    ),
    security(("jwt" = []))
)]
pub async fn delete(
    auth_user: AuthUser,
    State(state): State<Arc<AppState>>,
) -> Result<Json<ApiResponse<()>>, AppError> {
    auth_user.require_role(&[UserRole::Admin])?;
    let setting = state.setting_service.get_first().await?;
    state.setting_service.delete(setting.id).await?;
    Ok(Json(ApiResponse { data: () }))
}

