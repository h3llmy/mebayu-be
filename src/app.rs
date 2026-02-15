use std::sync::Arc;

use axum::{Router, middleware, routing::get};
use sqlx::PgPool;
use tower::ServiceBuilder;
use tower_http::trace::TraceLayer;

use crate::{
    core::{error::AppError, middleware::rate_limiter::rate_limiter_middleware},
    domain::{
        auth::service::AuthService, product_categories::service::ProductCategoryServiceImpl,
        products::service::ProductServiceImpl, users::service::UserServiceImpl,
    },
    infrastructure::{
        database::redis::create_redis_client,
        repository::{
            product_category_repository_impl::ProductCategoryRepositoryImpl,
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

pub async fn build_app(pool: PgPool, config: crate::core::config::Config) -> Router {
    let redis_client = create_redis_client(&config.redis_url);

    let product_repo = Arc::new(ProductRepositoryImpl::new(pool.clone()));
    let category_repo = Arc::new(ProductCategoryRepositoryImpl::new(pool.clone()));
    let user_repo = Arc::new(UserRepositoryImpl::new(pool));

    let product_service = Arc::new(ProductServiceImpl::new(product_repo));
    let product_category_service = Arc::new(ProductCategoryServiceImpl::new(category_repo));
    let user_service = Arc::new(UserServiceImpl::new(user_repo.clone()));
    let auth_service = Arc::new(AuthService::new(
        user_service.clone(),
        config.jwt_secret.clone(),
    ));

    let state = Arc::new(AppState {
        product_service,
        product_category_service,
        user_service,
        auth_service,
        redis_client,
        config: config.clone(),
    });

    let api_v1_router = Router::new()
        .nest("/auth", auth_routes())
        .nest("/products", product_routes())
        .nest("/product-categories", category_routes())
        .nest("/users", routes())
        .layer(middleware::from_fn_with_state(
            state.clone(),
            rate_limiter_middleware,
        ));

    Router::new()
        .route("/health", get(health_check))
        .nest("/api/v1", api_v1_router)
        .fallback(not_found)
        .layer(ServiceBuilder::new().layer(TraceLayer::new_for_http()))
        .with_state(state)
}
