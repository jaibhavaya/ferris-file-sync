pub mod migrations;
pub mod models;

use sqlx::postgres::PgPool;
use sqlx::postgres::PgPoolOptions;

pub async fn connect(database_url: &str) -> Result<PgPool, sqlx::Error> {
    let pool = PgPoolOptions::new().max_connections(5).connect(database_url).await?;

    Ok(pool)
}

pub async fn run_migrations(pool: &PgPool) -> Result<(), sqlx::Error> {
    sqlx::query(include_str!("../../migrations/20240324000001_create_files_table.sql"))
        .execute(pool)
        .await?;

    Ok(())
}
