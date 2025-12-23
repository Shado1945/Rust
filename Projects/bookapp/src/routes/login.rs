use crate::auth::token::create_jwt;
use crate::response::responses::Response;
use axum::{Json, Router, extract::State, routing::post};
use bcrypt::verify;
use chrono::{Duration, Utc};
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
    pub token: Option<String>,
    pub data: Option<UserData>,
}

#[derive(Debug, Serialize, FromRow)]
pub struct UserData {
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
            token: None,
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

    if !pwd_valid {
        error!(
            "LOGIN: Password provided was incorrect with this error {:?}",
            Response::Unauthorized
        );
        return Ok(Json(LoginResponse {
            code: Response::Unauthorized.status_code().as_u16(),
            message: "The password provided is incorrect".to_string(),
            token: None,
            data: None,
        }));
    }
    let secret = std::env::var("JWT_SECRET").unwrap_or_else(|_| "STAR_WARS_ROCKS".to_string());
    let token = match create_jwt(&username, &secret).await {
        Ok(t) => t,
        Err(e) => {
            error!("LOGIN: Failed to create JWT token: {:?}", e);
            return Err(Response::InternalError);
        }
    };
    let now = Utc::now();
    let expires_at = now + Duration::hours(8);

    match sqlx::query(UPSERT_USER_LOGIN)
        .bind(&username)
        .bind(&token)
        .bind(now)
        .bind(expires_at)
        .execute(&pool)
        .await
    {
        Ok(_) => {
            info!("LOGIN: Token saved for user '{}'", username);
        }
        Err(e) => {
            error!("LOGIN: Failed to save token to database: {:?}", e);
        }
    }

    Ok(Json(LoginResponse {
        code: Response::Success.status_code().as_u16(),
        message: "User logged in successfully".to_string(),
        token: Some(token),
        data: Some(user_data),
    }))
}

const FETCH_USER_DATA: &str = "
SELECT username
    ,name
    ,surname
    ,phone
    ,email
    ,pwd
FROM users
WHERE username = $1
";

const UPSERT_USER_LOGIN: &str = "
INSERT INTO user_login (username, token, created_datetime, expire_datetime)
VALUES ($1, $2, $3, $4)
ON CONFLICT (username)
DO UPDATE SET
    token = EXCLUDED.token,
    created_datetime = EXCLUDED.created_datetime,
    expire_datetime = EXCLUDED.expire_datetime
";
