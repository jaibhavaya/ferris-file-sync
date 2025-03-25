use axum::{extract::State, routing::get, Json, Router};
use sqlx::PgPool;

use crate::db::models::File;

pub fn create_router(pool: PgPool) -> Router {
    Router::new().route("/files", get(list_files)).with_state(pool)
}

async fn list_files(State(pool): State<PgPool>) -> Json<Vec<File>> {
    // Simple implementation to list files from the database
    let files = sqlx::query_as!(
        File,
        r#"
        SELECT id, name, created_at FROM files
        ORDER BY created_at DESC
        "#
    )
    .fetch_all(&pool)
    .await
    .unwrap_or_default();

    Json(files)
}
