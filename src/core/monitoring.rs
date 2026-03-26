use std::time::Instant;
use metrics::{histogram, counter};

pub async fn observe_db<F, T, E>(operation: &'static str, f: F) -> Result<T, E>
where
    F: std::future::Future<Output = Result<T, E>>,
{
    let start = Instant::now();
    let result = f.await;
    let latency = start.elapsed().as_secs_f64();
    
    let status = if result.is_ok() { "success" } else { "error" };
    let labels = [
        ("operation", operation),
        ("status", status),
    ];
    
    histogram!("db_query_duration_seconds", &labels).record(latency);
    counter!("db_queries_total", &labels).increment(1);
    
    result
}

pub async fn observe_redis<F, T, E>(operation: &'static str, f: F) -> Result<T, E>
where
    F: std::future::Future<Output = Result<T, E>>,
{
    let start = Instant::now();
    let result = f.await;
    let latency = start.elapsed().as_secs_f64();
    
    let status = if result.is_ok() { "success" } else { "error" };
    let labels = [
        ("command", operation),
        ("status", status),
    ];
    
    histogram!("redis_command_duration_seconds", &labels).record(latency);
    counter!("redis_commands_total", &labels).increment(1);
    
    result
}
