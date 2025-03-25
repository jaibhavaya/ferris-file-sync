mod api;
mod config;
mod db;

use std::net::SocketAddr;

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();

    let config = config::Config::from_env().expect("Failed to load config");

    let pool = db::connect(&config.database_url).await.expect("Failed to connect to database");

    db::run_migrations(&pool).await.expect("Failed to run migrations");

    let app = api::create_router(pool);

    let addr = SocketAddr::from(([0, 0, 0, 0], config.port));
    println!("Listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();

    axum::serve(listener, app).await.unwrap();
}
