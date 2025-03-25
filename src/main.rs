mod api;
mod config;
mod db;

use std::net::SocketAddr;

#[tokio::main]
async fn main() {
    // Load environment variables
    dotenv::dotenv().ok();

    // Load configuration
    let config = config::Config::from_env().expect("Failed to load config");

    // Set up database connection
    let pool = db::connect(&config.database_url).await.expect("Failed to connect to database");

    // Run migrations
    db::run_migrations(&pool).await.expect("Failed to run migrations");

    // Build the application
    let app = api::create_router(pool);

    // Run the server
    let addr = SocketAddr::from(([0, 0, 0, 0], config.port));
    println!("Listening on {}", addr);

    // Updated for Axum 0.7+
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
