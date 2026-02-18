use crate::{core::error::AppError, shared::app_state::AppState};
use axum::{
    extract::{ConnectInfo, State},
    http::Request,
    middleware::Next,
    response::Response,
};
use std::{net::SocketAddr, sync::Arc};

pub async fn rate_limiter_middleware(
    State(state): State<Arc<AppState>>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    request: Request<axum::body::Body>,
    next: Next,
) -> Result<Response, AppError> {
    let mut conn = state
        .redis_client
        .get_multiplexed_async_connection()
        .await
        .map_err(|e| {
            tracing::error!("Redis error: {}", e);
            AppError::Database("Redis error".to_string())
        })?;

    let ip = addr.ip().to_string();
    let key = format!("rate_limit:{}", ip);

    let (count, _): (i64, i64) = redis::pipe()
        .atomic()
        .incr(&key, 1)
        .expire(&key, state.config.rate_limit_window as i64)
        .query_async(&mut conn)
        .await
        .map_err(|e| {
            tracing::error!("Redis error: {}", e);
            AppError::Database("Redis error".to_string())
        })?;

    if count > state.config.rate_limit_requests as i64 {
        tracing::warn!("Too many requests from {}: {}", ip, count);
        return Err(AppError::TooManyRequests("Too many requests".to_string()));
    }

    Ok(next.run(request).await)
}
