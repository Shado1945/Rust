use crate::response::responses::Response;
use axum::{Json, Router, extract::State, routing::post};
use bcrypt::verify;
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool};
use tracing::{error, info};

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    username: String,
    password: String,
}

#[derive(Debug, Serialize)]
pub struct LoginResponse {
    pub code: u16,
    pub message: String,
    pub data: Option<UserData>,
}

#[derive(Debug, Serialize, FromRow)]
pub struct UserData {
    pub id: i32,
    pub username: String,
    pub name: String,
    pub surname: String,
    pub phone: String,
    pub email: String,
    #[serde(skip_serializing)]
    pub pwd: String,
}

pub fn login_route(pool: PgPool) -> Router {
    Router::new().route("/login", post(login)).with_state(pool)
}

pub async fn login(
    State(pool): State<PgPool>,
    Json(payload): Json<LoginRequest>,
) -> Result<Json<LoginResponse>, Response> {
    let LoginRequest { username, password } = payload;
    let user: Option<UserData> = match sqlx::query_as::<_, UserData>(FETCH_USER_DATA)
        .bind(&username)
        .fetch_optional(&pool)
        .await
    {
        Ok(r) => r,
        Err(_) => {
            error!(
                "LOGIN: Failed to fetch user data with error: {:?}",
                Response::InternalError
            );
            return Err(Response::InternalError);
        }
    };

    let user_data = if let Some(user) = user {
        user
    } else {
        error!(
            "LOGIN: Username provided was incorrect with this error {:?}",
            Response::Unauthorized
        );
        return Ok(Json(LoginResponse {
            code: Response::Unauthorized.status_code().as_u16(),
            message: "The username provided is incorrect".to_string(),
            data: None,
        }));
    };

    let pwd_valid = match verify(password, user_data.pwd.as_str()) {
        Ok(valid) => valid,
        Err(_) => {
            error!(
                "LOGIN: Password verification failed with error {:?}",
                Response::InternalError
            );
            return Err(Response::InternalError);
        }
    };

    if pwd_valid {
        info!("LOGIN: User {username} successfully logged in");
        Ok(Json(LoginResponse {
            code: Response::Success.status_code().as_u16(),
            message: "User logged in successfuly".to_string(),
            data: Some(user_data),
        }))
    } else {
        error!(
            "LOGIN: Password provided was incorrect with this error {:?}",
            Response::Unauthorized
        );
        Ok(Json(LoginResponse {
            code: Response::Unauthorized.status_code().as_u16(),
            message: "The password provided is incorrect".to_string(),
            data: None,
        }))
    }
}

const FETCH_USER_DATA: &str = "
SELECT id
    ,username
    ,name
    ,surname
    ,phone
    ,email
    ,pwd
FROM users
WHERE username = $1
";
