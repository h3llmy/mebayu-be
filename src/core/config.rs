use std::env;

#[derive(Clone, Debug)]
pub struct Config {
    pub host: String,
    pub port: String,
    pub database_url: String,
    pub redis_url: String,
    pub rate_limit_requests: u64,
    pub rate_limit_window: u64,
    pub jwt_secret: String,
}

impl Config {
    pub fn from_env() -> Self {
        Self {
            host: env::var("HOST").unwrap_or_else(|_| "127.0.0.1".to_string()),
            port: env::var("PORT").unwrap_or_else(|_| "3000".to_string()),

            database_url: env::var("DATABASE_URL").expect("DATABASE_URL must be set"),

            redis_url: env::var("REDIS_URL")
                .unwrap_or_else(|_| "redis://127.0.0.1:6379".to_string()),
            rate_limit_requests: env::var("RATE_LIMIT_REQUESTS")
                .unwrap_or_else(|_| "100".to_string())
                .parse()
                .unwrap_or(100),
            rate_limit_window: env::var("RATE_LIMIT_WINDOW")
                .unwrap_or_else(|_| "60".to_string())
                .parse()
                .unwrap_or(60),
            jwt_secret: env::var("JWT_SECRET").unwrap_or_else(|_| "default_secret".to_string()),
        }
    }
}
