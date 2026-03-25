use axum::{
    body::Body,
    extract::Request,
    http::{HeaderValue, Response},
    middleware::Next,
};
use http_body_util::BodyExt;
use std::time::Instant;
use tracing::Instrument;
use uuid::Uuid;

pub async fn track_metrics(req: Request, next: Next) -> Response<Body> {
    // Generate or reuse request id
    let request_id = req
        .headers()
        .get("x-request-id")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string())
        .unwrap_or_else(|| Uuid::new_v4().to_string());

    let (parts, body) = req.into_parts();
    
    // Extract and mask query parameters
    let query = parts.uri.query().unwrap_or("").to_string();
    let masked_query = if query.is_empty() {
        "".to_string()
    } else {
        mask_query(&query)
    };

    // Read and mask request body
    let body_bytes = match body.collect().await {
        Ok(collected) => collected.to_bytes(),
        Err(_) => axum::body::Bytes::new(),
    };

    let body_str = String::from_utf8_lossy(&body_bytes).to_string();
    let (masked_body, maybe_json) = if body_str.trim().starts_with('{') || body_str.trim().starts_with('[') {
        (mask_json_body(&body_str), true)
    } else {
        (body_str.clone(), false)
    };

    // Create a new request based on parts and the body we read
    let mut req = Request::from_parts(parts, Body::from(body_bytes.clone()));

    // Create a span for this request that includes the request_id and masked request info
    let span = tracing::info_span!(
        "http_request",
        %request_id,
        method = %req.method(),
        uri = %req.uri().path(),
        query = %masked_query,
        body = %if maybe_json { masked_body.clone() } else { "Binary or Non-JSON Body".to_string() },
    );

    let start = Instant::now();

    // Store in request extensions so handlers can access it
    req.extensions_mut().insert(request_id.clone());

    let path = if let Some(matched_path) = req.extensions().get::<axum::extract::MatchedPath>() {
        matched_path.as_str().to_string()
    } else {
        req.uri().path().to_string()
    };

    let method = req.method().clone();

    // Run the next handler within the created span
    let mut response = next.run(req).instrument(span.clone()).await;

    // Add request id to response header
    response
        .headers_mut()
        .insert("x-request-id", HeaderValue::from_str(&request_id).unwrap());

    let latency = start.elapsed().as_secs_f64();
    let status = response.status().as_u16().to_string();

    let labels = [
        ("method", method.to_string()),
        ("path", path.clone()),
        ("status", status.clone()),
        ("request_id", request_id),
    ];

    metrics::counter!("http_requests_total", &labels).increment(1);
    metrics::histogram!("http_request_duration_seconds", &labels).record(latency);

    // Log the completion of the request within the same span
    let _enter = span.enter();
    tracing::info!(
        path = %path,
        status = %status,
        latency = ?start.elapsed(),
        "HTTP Request Completed"
    );

    response
}

fn mask_query(query: &str) -> String {
    query
        .split('&')
        .map(|pair| {
            let mut parts = pair.splitn(2, '=');
            let key = parts.next().unwrap_or("");
            let val = parts.next().is_some();
            if is_sensitive_key(key) && val {
                format!("{}={}", key, "********")
            } else {
                pair.to_string()
            }
        })
        .collect::<Vec<_>>()
        .join("&")
}

fn mask_json_body(body_str: &str) -> String {
    if let Ok(mut value) = serde_json::from_str::<serde_json::Value>(body_str) {
        mask_sensitive_json(&mut value);
        return value.to_string();
    }
    body_str.to_string()
}

fn mask_sensitive_json(value: &mut serde_json::Value) {
    match value {
        serde_json::Value::Object(map) => {
            for (k, v) in map.iter_mut() {
                if is_sensitive_key(k) {
                    *v = serde_json::Value::String("********".to_string());
                } else {
                    mask_sensitive_json(v);
                }
            }
        }
        serde_json::Value::Array(arr) => {
            for v in arr.iter_mut() {
                mask_sensitive_json(v);
            }
        }
        _ => {}
    }
}

fn is_sensitive_key(key: &str) -> bool {
    let k = key.to_lowercase();
    k.contains("password") || k.contains("token") || k.contains("secret") || k.contains("key")
}
