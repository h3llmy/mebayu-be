use std::env;
use tracing_subscriber::{filter::EnvFilter, prelude::*};
use opentelemetry::trace::TracerProvider as _;
use opentelemetry_otlp::WithExportConfig;
use opentelemetry_sdk::trace::TracerProvider as SdkTracerProvider;

#[derive(Clone, Debug, Default)]
pub struct Config {
    pub host: String,
    pub port: String,
    pub database_url: String,
    pub redis_url: String,
    pub rate_limit_requests: u64,
    pub rate_limit_window: u64,
    pub jwt_secret: String,
    pub s3_endpoint: String,
    pub s3_region: String,
    pub s3_bucket: String,
    pub s3_access_key: String,
    pub s3_secret_key: String,
    pub admin_email: String,
    pub admin_username: String,
    pub admin_password: String,
    pub default_setting_email: String,
    pub default_setting_whatsapp: String,
    pub default_setting_hero_images: Vec<String>,
}

impl Config {
    pub fn from_env() -> Self {
        Self {
            // app setup
            host: env::var("HOST").unwrap_or_else(|_| "127.0.0.1".to_string()),
            port: env::var("PORT").unwrap_or_else(|_| "3000".to_string()),

            // database
            database_url: env::var("DATABASE_URL").expect("DATABASE_URL must be set"),

            // reddis
            redis_url: env::var("REDIS_URL")
                .unwrap_or_else(|_| "redis://127.0.0.1:6379".to_string()),

            // rate limiter
            rate_limit_requests: env::var("RATE_LIMIT_REQUESTS")
                .unwrap_or_else(|_| "100".to_string())
                .parse()
                .unwrap_or(100),
            rate_limit_window: env::var("RATE_LIMIT_WINDOW")
                .unwrap_or_else(|_| "60".to_string())
                .parse()
                .unwrap_or(60),

            // jwt
            jwt_secret: env::var("JWT_SECRET").unwrap_or_else(|_| "default_secret".to_string()),

            // s3 object storage
            s3_endpoint: env::var("S3_ENDPOINT")
                .unwrap_or_else(|_| "http://localhost:9000".to_string()),
            s3_region: env::var("S3_REGION").unwrap_or_else(|_| "us-east-1".to_string()),
            s3_bucket: env::var("S3_BUCKET").expect("S3_BUCKET must be set"),
            s3_access_key: env::var("S3_ACCESS_KEY").expect("S3_ACCESS_KEY must be set"),
            s3_secret_key: env::var("S3_SECRET_KEY").expect("S3_SECRET_KEY must be set"),

            // initial admin
            admin_email: env::var("ADMIN_EMAIL")
                .unwrap_or_else(|_| "admin@example.com".to_string()),
            admin_username: env::var("ADMIN_USERNAME").unwrap_or_else(|_| "admin".to_string()),
            admin_password: env::var("ADMIN_PASSWORD").unwrap_or_else(|_| "admin".to_string()),

            // default settings
            default_setting_email: env::var("DEFAULT_SETTING_EMAIL").unwrap_or_else(|_| "mebayu@admin.com".to_string()),
            default_setting_whatsapp: env::var("DEFAULT_SETTING_WHATSAPP").unwrap_or_else(|_| "628123456789".to_string()),
            default_setting_hero_images: env::var("DEFAULT_SETTING_HERO_IMAGES")
                .unwrap_or_else(|_| "http://localhost:9000/mebayu/hero1.jpg,http://localhost:9000/mebayu/hero2.jpg".to_string())
                .split(',')
                .map(|s| s.to_string())
                .collect(),
        }
    }

    pub fn logger_setup() {
        let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

        let otlp_endpoint = std::env::var("OTEL_EXPORTER_OTLP_ENDPOINT")
            .unwrap_or_else(|_| "http://alloy:4317".to_string());

        let exporter = opentelemetry_otlp::SpanExporter::builder()
            .with_tonic()
            .with_endpoint(otlp_endpoint)
            .build()
            .expect("Error initializing OTLP tracer");

        let provider = SdkTracerProvider::builder()
            .with_batch_exporter(exporter, opentelemetry_sdk::runtime::Tokio)
            .with_resource(opentelemetry_sdk::Resource::new(vec![
                opentelemetry::KeyValue::new("service.name", "mebayu-backend"),
            ]))
            .build();

        opentelemetry::global::set_tracer_provider(provider.clone());
        let tracer = provider.tracer("mebayu-backend");

        let telemetry = tracing_opentelemetry::layer().with_tracer(tracer);

        tracing_subscriber::registry()
            .with(filter)
            .with(telemetry)
            .with(tracing_subscriber::fmt::layer().compact().with_target(true))
            .init();
    }
}
