use crate::shared::app_state::AppState;
use axum::{
    extract::{ConnectInfo, State},
    http::{Request, StatusCode},
    middleware::Next,
    response::Response,
};
use std::{net::SocketAddr, sync::Arc};

pub async fn rate_limiter_middleware(
    State(state): State<Arc<AppState>>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    request: Request<axum::body::Body>,
    next: Next,
) -> Result<Response, StatusCode> {
    let mut conn = state
        .redis_client
        .get_multiplexed_async_connection()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let ip = addr.ip().to_string();
    let key = format!("rate_limit:{}", ip);

    let (count, _): (i64, i64) = redis::pipe()
        .atomic()
        .incr(&key, 1)
        .expire(&key, state.config.rate_limit_window as i64)
        .query_async(&mut conn)
        .await
        .map_err(|e| {
            eprintln!("Redis error: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    if count > state.config.rate_limit_requests as i64 {
        return Err(StatusCode::TOO_MANY_REQUESTS);
    }

    Ok(next.run(request).await)
}
