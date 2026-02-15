use sqlx::PgPool;

pub async fn run_migrations(pool: &PgPool) {
    sqlx::migrate!("./src/infrastructure/database/migration")
        .run(pool)
        .await
        .expect("Migration failed");
}
