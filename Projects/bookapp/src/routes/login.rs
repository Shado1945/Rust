use axum::{Json, Router, extract::State, routing::post};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool};
use tracing::{error, info};

use crate::auth::PasswordManager;
use crate::config::ArgonConfig;
use crate::response::responses::Response;

#[derive(Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Serialize)]
pub struct LoginResponse {
    pub code: u16,
    pub message: String,
    pub id: Option<i32>,
    pub username: Option<String>,
    pub name: Option<String>,
    pub surname: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
}

#[derive(Debug, FromRow)]
struct UserRecord {
    id: i32,
    username: String,
    name: String,
    surname: String,
    email: String,
    phone: String,
    pwd: String,
}

pub fn login_route(pool: PgPool) -> Router {
    Router::new().route("/login", post(login)).with_state(pool)
}

async fn login(
    State(pool): State<PgPool>,
    Json(payload): Json<LoginRequest>,
) -> Result<Json<LoginResponse>, Response> {
    let LoginRequest { username, password } = payload;

    let user: Option<UserRecord> = match sqlx::query_as(GET_USER)
        .bind(username)
        .fetch_optional(&pool)
        .await
    {
        Ok(r) => r,
        Err(_) => {
            error!(
                "LOGIN: Login user failed with following error: {:?}",
                Response::InternalError
            );
            return Err(Response::InternalError);
        }
    };

    let config = ArgonConfig::from_env();

    let password_manager = match PasswordManager::new(config) {
        Ok(pm) => pm,
        Err(e) => {
            error!("LOGIN: Failed to create password manager: {}", e);
            return Err(Response::InternalError);
        }
    };

    match user {
        Some(user) => match password_manager.verify_password(&password, &user.pwd).await {
            Ok(true) => {
                info!("LOGIN: User: {} logged in successfully", user.username);
                Ok(Json(LoginResponse {
                    code: Response::Success.status_code().as_u16(),
                    message: "Login successful".to_string(),
                    id: Some(user.id),
                    username: Some(user.username),
                    name: Some(user.name),
                    surname: Some(user.surname),
                    email: Some(user.email),
                    phone: Some(user.phone),
                }))
            }
            Ok(false) => {
                error!("LOGIN: Invalid password for user {}", user.username);
                Ok(Json(LoginResponse {
                    code: Response::Unauthorized.status_code().as_u16(),
                    message: "Invalid credentials provided".to_string(),
                    id: None,
                    username: None,
                    name: None,
                    surname: None,
                    email: None,
                    phone: None,
                }))
            }
            Err(e) => {
                error!(
                    "LOGIN: Password verification error with error {:?}",
                    Response::InternalError
                );
                return Err(Response::InternalError);
            }
        },
        None => {
            error!("LOGIN: Login user not found");
            Ok(Json(LoginResponse {
                code: Response::Unauthorized.status_code().as_u16(),
                message: "Invalid credentials provided".to_string(),
                id: None,
                username: None,
                name: None,
                surname: None,
                email: None,
                phone: None,
            }))
        }
    }
}

const GET_USER: &str = "
SELECT id
    ,username
    ,name
    ,surname
    ,email
    ,phone
    ,pwd
FROM users
WHERE username = $1
";
