pub mod encryption;
pub mod migrations;
pub mod models;
pub mod onedrive;

use sqlx::postgres::PgPool;
use sqlx::postgres::PgPoolOptions;

pub async fn connect(database_url: &str) -> Result<PgPool, sqlx::Error> {
    let pool = PgPoolOptions::new().max_connections(5).connect(database_url).await?;

    Ok(pool)
}

pub async fn run_migrations(pool: &PgPool) -> Result<(), sqlx::Error> {
    // Use SQLx's built-in migration instead of manual queries
    sqlx::migrate!("./migrations")
        .run(pool)
        .await?;
    
    Ok(())
}
