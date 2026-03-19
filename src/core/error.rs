use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::Serialize;
use std::collections::HashMap;
use utoipa::ToSchema;

#[derive(Debug)]
pub enum AppError {
    NotFound(String),
    TooManyRequests(String),
    Validation(HashMap<String, Vec<String>>),
    Database(String),
    Unauthorized(String),
    Forbidden(String),
    Internal(String),
    Storage(String),
}

#[derive(Serialize, ToSchema)]
pub struct ErrorResponse {
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    errors: Option<HashMap<String, Vec<String>>>,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message, errors) = match &self {
            AppError::NotFound(msg) => (StatusCode::NOT_FOUND, msg.clone(), None),
            AppError::TooManyRequests(msg) => (StatusCode::TOO_MANY_REQUESTS, msg.clone(), None),
            AppError::Validation(errs) => (
                StatusCode::UNPROCESSABLE_ENTITY,
                "Validation Failed".to_string(),
                Some(errs.clone()),
            ),
            AppError::Database(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg.clone(), None),
            AppError::Unauthorized(msg) => (StatusCode::UNAUTHORIZED, msg.clone(), None),
            AppError::Forbidden(msg) => (StatusCode::FORBIDDEN, msg.clone(), None),
            AppError::Internal(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg.clone(), None),
            AppError::Storage(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg.clone(), None),
        };

        if status.is_server_error() {
            tracing::error!(
                status = %status,
                message = %message,
                error = ?self,
                "Server error occurred"
            );
        } else {
            tracing::warn!(
                status = %status,
                message = %message,
                error = ?self,
                "Client error occurred"
            );
        }

        (status, Json(ErrorResponse { message, errors })).into_response()
    }
}
