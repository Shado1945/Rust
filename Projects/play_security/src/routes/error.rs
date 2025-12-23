use crate::response::responses::Response;
use axum::{body::Body, http::Request};
use tracing::warn;

pub async fn error_route(req: Request<Body>) -> Response {
    warn!("{} - {}", Response::NotFound.message(), req.uri().path());
    Response::NotFound
}
