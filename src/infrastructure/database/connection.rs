use sqlx::{PgPool, postgres::PgPoolOptions};
use tracing::{error, info};

pub async fn create_pool(database_url: &str) -> Result<PgPool, sqlx::Error> {
    match PgPoolOptions::new()
        .max_connections(5)
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
