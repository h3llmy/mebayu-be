use std::time::Duration;

use sqlx::{PgPool, postgres::PgPoolOptions};
use tracing::{error, info};

pub async fn create_pool(database_url: &str) -> Result<PgPool, sqlx::Error> {
    match PgPoolOptions::new()
        .max_connections(20)
        .min_connections(5)
        .acquire_timeout(Duration::from_secs(3))
        .max_lifetime(Duration::from_mins(30))
        .idle_timeout(Duration::from_mins(10))
        .test_before_acquire(true)
        .connect(database_url)
        .await
    {
        Ok(pool) => {
            info!("✅ Database connected successfully");
            Ok(pool)
        }
        Err(e) => {
            error!("❌ Failed to connect to database: {:?}", e);
            Err(e)
        }
    }
}
