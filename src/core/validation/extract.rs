use axum::{
    extract::{FromRequest, FromRequestParts, Json, Query, Request},
    http::request::Parts,
};
use serde::de::DeserializeOwned;
use std::collections::HashMap;
use validator::{Validate, ValidationErrors};

use crate::core::error::AppError;

//
// ============================================================
// JSON VALIDATION
// ============================================================
//

pub struct ValidatedJson<T>(pub T);

impl<S, T> FromRequest<S> for ValidatedJson<T>
where
    T: DeserializeOwned + Validate,
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        let Json(value) = Json::<T>::from_request(req, state)
            .await
            .map_err(handle_json_rejection)?;

        value.validate().map_err(map_validation_errors)?;

        Ok(ValidatedJson(value))
    }
}

fn handle_json_rejection(err: axum::extract::rejection::JsonRejection) -> AppError {
    use axum::extract::rejection::JsonRejection::*;

    let mut errors = HashMap::new();

    match err {
        MissingJsonContentType(_) => {
            errors.insert(
                "body".to_string(),
                vec!["Content-Type must be application/json".to_string()],
            );
        }

        JsonSyntaxError(_) => {
            errors.insert("body".to_string(), vec!["Malformed JSON".to_string()]);
        }

        JsonDataError(e) => {
            errors.insert(
                "body".to_string(),
                vec![format!("Invalid request data: {}", e)],
            );
        }

        BytesRejection(_) => {
            errors.insert(
                "body".to_string(),
                vec!["Failed to read request body".to_string()],
            );
        }

        _ => {
            errors.insert("body".to_string(), vec!["Invalid request body".to_string()]);
        }
    }

    AppError::Validation(errors)
}

//
// ============================================================
// QUERY VALIDATION
// ============================================================
//

pub struct ValidatedQuery<T>(pub T);

impl<S, T> FromRequestParts<S> for ValidatedQuery<T>
where
    T: DeserializeOwned + Validate,
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let Query(value) = Query::<T>::from_request_parts(parts, state)
            .await
            .map_err(handle_query_rejection)?;

        value.validate().map_err(map_validation_errors)?;

        Ok(ValidatedQuery(value))
    }
}

fn handle_query_rejection(err: axum::extract::rejection::QueryRejection) -> AppError {
    let mut errors = HashMap::new();

    errors.insert(
        "query".to_string(),
        vec![format!("Invalid query parameters: {}", err)],
    );

    AppError::Validation(errors)
}

//
// ============================================================
// SHARED VALIDATION ERROR MAPPER
// ============================================================
//

fn map_validation_errors(err: ValidationErrors) -> AppError {
    let mut errors = HashMap::new();

    for (field, field_errors) in err.field_errors() {
        let messages: Vec<String> = field_errors
            .iter()
            .map(|e| {
                e.message
                    .as_ref()
                    .map(|m| m.to_string())
                    .unwrap_or_else(|| format!("{} is invalid", field))
            })
            .collect();

        errors.insert(field.to_string(), messages);
    }

    AppError::Validation(errors)
}
