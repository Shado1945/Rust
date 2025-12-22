mod auth;
mod config;
mod database;
mod response;
mod routes;

use sqlx::PgPool;
use std::net::SocketAddr;
use tokio::net::TcpListener;
use tracing::info;

#[tokio::main]
async fn main() {
    dotenv::dotenv().expect("Failed to load .env file");
    tracing_subscriber::fmt()
        .with_target(false)
        .compact()
        .init();

    info!("Starting server...");
    info!("Initializing Database");
    //Database
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    let pool: PgPool = database::create_pool(&database_url)
        .await
        .expect("Failed to connect to database");

    info!("Database connection established");
    //tables
    database::models::Tables::initialize_tables(&pool)
        .await
        .expect("Failed to initialize tables");

    info!("Tables initialized successfully");

    //Routes
    let app = routes::home::home_route()
        .merge(routes::users::users_route(pool.clone()))
        .merge(routes::login::login_route(pool.clone()))
        .fallback(routes::error::error_route);
    info!("Routes initialized successfully");

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    let listener = TcpListener::bind(addr).await.unwrap();
    info!("Server listening on port {}", addr.port());
    info!("Server running on http://{}", addr);

    axum::serve(listener, app.into_make_service())
        .await
        .unwrap();
}
