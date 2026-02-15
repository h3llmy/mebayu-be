use axum::{
    body::to_bytes,
    extract::{FromRequest, FromRequestParts, Query, Request},
    http::request::Parts,
};
use serde::de::DeserializeOwned;
use std::collections::HashMap;
use validator::{Validate, ValidationErrors};

use crate::core::error::AppError;

//
// -----------------------------
// JSON VALIDATION
// -----------------------------
//

pub struct ValidatedJson<T>(pub T);

impl<S, T> FromRequest<S> for ValidatedJson<T>
where
    T: DeserializeOwned + Validate + 'static,
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request(req: Request, _state: &S) -> Result<Self, Self::Rejection> {
        // 1MB limit
        let bytes = to_bytes(req.into_body(), 1024 * 1024)
            .await
            .map_err(|err| {
                let mut errors = HashMap::new();
                errors.insert("body".to_string(), vec![err.to_string()]);
                AppError::Validation(errors)
            })?;

        // Use serde_path_to_error for better error path tracking
        let mut deserializer = serde_json::Deserializer::from_slice(&bytes);

        let value: T = serde_path_to_error::deserialize(&mut deserializer).map_err(|err| {
            let mut errors = HashMap::new();

            let path = err.path().to_string();
            let inner_err = err.into_inner();
            let msg = inner_err.to_string();

            let clean_msg = clean_serde_error(&path, &msg);

            errors.insert(
                if path.is_empty() {
                    "body".to_string()
                } else {
                    path
                },
                vec![clean_msg],
            );

            AppError::Validation(errors)
        })?;

        value.validate().map_err(map_validation_errors)?;

        Ok(ValidatedJson(value))
    }
}

//
// -----------------------------
// QUERY VALIDATION
// -----------------------------
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
            .map_err(|err| {
                let mut errors = HashMap::new();
                errors.insert("query".to_string(), vec![err.to_string()]);
                AppError::Validation(errors)
            })?;

        value.validate().map_err(map_validation_errors)?;

        Ok(ValidatedQuery(value))
    }
}

//
// -----------------------------
// SHARED HELPERS
// -----------------------------
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

fn clean_serde_error(path: &str, msg: &str) -> String {
    if msg.contains("missing field") {
        format!("{} is required", capitalize(path))
    } else if msg.contains("invalid type") {
        if let Some(expected) = msg.split(", expected ").nth(1) {
            let type_info = expected.split(" at line ").next().unwrap_or(expected);
            format!("Invalid data type. Expected {}", type_info)
        } else {
            "Invalid data type".to_string()
        }
    } else {
        msg.to_string()
    }
}

fn capitalize(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        None => String::new(),
        Some(f) => f.to_uppercase().collect::<String>() + chars.as_str(),
    }
}
