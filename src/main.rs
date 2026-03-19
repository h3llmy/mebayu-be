mod app;
mod core;
mod domain;
mod infrastructure;
mod presentation;
mod shared;

use crate::core::config::Config;

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    let config = Config::from_env();
    Config::logger_setup();

    let app = app::build_app(config.clone()).await;
    let addr = format!("{}:{}", config.host, config.port);

    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();

    tracing::info!(
        "Server running on http://{}",
        listener.local_addr().unwrap()
    );

    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<std::net::SocketAddr>(),
    )
    .await
    .unwrap();
}
