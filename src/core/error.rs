use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::Serialize;
use std::collections::HashMap;

#[derive(Debug)]
pub enum AppError {
    NotFound(String),
    Validation(HashMap<String, Vec<String>>),
    Database(String),
    Unauthorized(String),
    Forbidden(String),
}

#[derive(Serialize)]
struct ErrorResponse {
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    errors: Option<HashMap<String, Vec<String>>>,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message, errors) = match self {
            AppError::NotFound(msg) => (StatusCode::NOT_FOUND, msg, None),
            AppError::Validation(errs) => (
                StatusCode::UNPROCESSABLE_ENTITY,
                "Validation Failed".to_string(),
                Some(errs),
            ),
            AppError::Database(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg, None),
            AppError::Unauthorized(msg) => (StatusCode::UNAUTHORIZED, msg, None),
            AppError::Forbidden(msg) => (StatusCode::FORBIDDEN, msg, None),
        };

        (status, Json(ErrorResponse { message, errors })).into_response()
    }
}
