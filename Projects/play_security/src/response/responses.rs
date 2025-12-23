use axum::http::StatusCode;
use axum::response::{IntoResponse, Response as AxumResponse};
use jsonwebtoken::errors::Error as JwtError;

#[derive(Debug, Clone)]
pub enum Response {
    Success,
    NotFound,
    NoUserFound,
    InternalError,
    BadRequest,
    Unauthorized,
    // Forbidden,
    InvalidToken(String),
}

impl Response {
    pub const fn status_code(&self) -> StatusCode {
        match self {
            Response::Success => StatusCode::OK,
            Response::NotFound => StatusCode::NOT_FOUND,
            Response::NoUserFound => StatusCode::NOT_FOUND,
            Response::InternalError => StatusCode::INTERNAL_SERVER_ERROR,
            Response::BadRequest => StatusCode::BAD_REQUEST,
            Response::Unauthorized => StatusCode::UNAUTHORIZED,
            // Response::Forbidden => StatusCode::FORBIDDEN,
            Response::InvalidToken(_) => StatusCode::UNAUTHORIZED,
        }
    }

    pub fn message(&self) -> String {
        match self {
            Response::Success => "Success".to_string(),
            Response::NotFound => "Not Found".to_string(),
            Response::NoUserFound => "No User data Found".to_string(),
            Response::InternalError => "Internal Server Error".to_string(),
            Response::BadRequest => "Bad Request".to_string(),
            Response::Unauthorized => "Unauthorized".to_string(),
            // Response::Forbidden => "Forbidden",
            Response::InvalidToken(msg) => format!("Token Invalid: {}", msg),
        }
    }
}

impl IntoResponse for Response {
    fn into_response(self) -> AxumResponse {
        let body = serde_json::json!({
            "code": self.status_code().as_u16(),
            "message": self.message()
        });

        (self.status_code(), axum::Json(body)).into_response()
    }
}

impl From<JwtError> for Response {
    fn from(err: JwtError) -> Self {
        Response::InvalidToken(err.to_string())
    }
}
