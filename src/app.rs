use std::sync::Arc;

use axum::{
    Router,
    http::{Method, header},
    middleware,
    routing::get,
};
use metrics_exporter_prometheus::PrometheusBuilder;
use sqlx::PgPool;
use tower::ServiceBuilder;
use tower_http::{
    compression::CompressionLayer,
    cors::{Any, CorsLayer},
    trace::TraceLayer,
};
use utoipa::OpenApi;

use crate::{
    core::{
        config::Config,
        error::AppError,
        middleware::{metrics, rate_limiter::rate_limiter_middleware},
    },
    domain::{
        auth::service::AuthService, product_categories::service::ProductCategoryServiceImpl,
        product_materials::service::ProductMaterialServiceImpl,
        products::service::ProductServiceImpl, users::service::UserServiceImpl,
    },
    infrastructure::{
        database::redis::create_redis_client,
        object_storage::s3::S3Service,
        repository::{
            product_category_repository_impl::ProductCategoryRepositoryImpl,
            product_material_repository_impl::ProductMaterialRepositoryImpl,
            product_repository_impl::ProductRepositoryImpl,
            user_repository_impl::UserRepositoryImpl,
        },
    },
    presentation::http::*,
    shared::app_state::AppState,
};

async fn not_found() -> AppError {
    AppError::NotFound("Resource Not Found".to_string())
}

async fn health_check() -> Result<String, AppError> {
    Ok("OK".to_string())
}

pub async fn build_app(pool: PgPool, config: Config) -> Router {
    let redis_client = create_redis_client(&config.redis_url);

    let product_repo = Arc::new(ProductRepositoryImpl::new(pool.clone()));
    let category_repo = Arc::new(ProductCategoryRepositoryImpl::new(pool.clone()));
    let material_repo = Arc::new(ProductMaterialRepositoryImpl::new(pool.clone()));
    let user_repo = Arc::new(UserRepositoryImpl::new(pool));

    let s3_service = Arc::new(S3Service::new(&config).await);
    let product_service = Arc::new(ProductServiceImpl::new(product_repo, s3_service.clone()));
    let product_category_service = Arc::new(ProductCategoryServiceImpl::new(category_repo));
    let product_material_service = Arc::new(ProductMaterialServiceImpl::new(material_repo));
    let user_service = Arc::new(UserServiceImpl::new(user_repo.clone(), config.clone()));
    let auth_service = Arc::new(AuthService::new(
        user_service.clone(),
        config.jwt_secret.clone(),
    ));

    user_service.create_initial_user().await;

    let state = Arc::new(AppState {
        product_service,
        product_category_service,
        product_material_service,
        user_service,
        auth_service,
        redis_client,
        s3_service,
        config: config.clone(),
    });

    let api_v1_router = Router::new()
        .nest("/auth", auth_routes())
        .nest("/products", product_routes())
        .nest("/product-categories", category_routes())
        .nest("/product-materials", product_material_routes())
        .nest("/users", routes())
        .nest("/storages", storage_routes())
        .layer(middleware::from_fn_with_state(
            state.clone(),
            rate_limiter_middleware,
        ));

    Router::new()
        .nest("/api/v1", api_v1_router)
        .fallback(not_found)
        .route("/health", get(health_check))
        .route(
            "/metrics",
            get(move || {
                std::future::ready(
                    PrometheusBuilder::new()
                        .install_recorder()
                        .expect("failed to install Prometheus recorder")
                        .render(),
                )
            }),
        )
        .merge(
            utoipa_swagger_ui::SwaggerUi::new("/swagger-ui")
                .url("/api-docs/openapi.json", openapi::ApiDoc::openapi()),
        )
        .layer(
            ServiceBuilder::new()
                .layer(TraceLayer::new_for_http())
                .layer(middleware::from_fn(metrics::track_metrics)),
        )
        .layer(
            CompressionLayer::new()
                .gzip(true)
                .br(true)
                .deflate(true)
                .zstd(true),
        )
        .layer(
            CorsLayer::new()
                .allow_origin(Any) // ⚠️ allow all origins (change in production)
                .allow_methods([
                    Method::GET,
                    Method::POST,
                    Method::PUT,
                    Method::DELETE,
                    Method::OPTIONS,
                ])
                .allow_headers([header::AUTHORIZATION, header::CONTENT_TYPE]),
        )
        .with_state(state)
}
