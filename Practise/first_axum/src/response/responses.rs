use axum::http::StatusCode;
use axum::response::{IntoResponse, Response as AxumResponse};

#[derive(Debug, Clone, Copy)]
pub enum Response {
    Success,
    NotFound,
    NoUserFound,
    InternalError,
    BadRequest,
    // Unauthorized,
    // Forbidden,
}

impl Response {
    pub const fn status_code(&self) -> StatusCode {
        match self {
            Response::Success => StatusCode::OK,
            Response::NotFound => StatusCode::NOT_FOUND,
            Response::NoUserFound => StatusCode::NOT_FOUND,
            Response::InternalError => StatusCode::INTERNAL_SERVER_ERROR,
            Response::BadRequest => StatusCode::BAD_REQUEST,
            // Response::Unauthorized => StatusCode::UNAUTHORIZED,
            // Response::Forbidden => StatusCode::FORBIDDEN,
        }
    }

    pub const fn message(&self) -> &'static str {
        match self {
            Response::Success => "Success",
            Response::NotFound => "Not Found",
            Response::NoUserFound => "No User data Found",
            Response::InternalError => "Internal Server Error",
            Response::BadRequest => "Bad Request",
            // Response::Unauthorized => "Unauthorized",
            // Response::Forbidden => "Forbidden",
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
