use axum::{Router, routing::get};

pub fn home_route() -> Router {
    Router::new().route("/", get(root))
}

async fn root() -> &'static str {
    "Hello Axum"
}
