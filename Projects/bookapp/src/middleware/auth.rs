use crate::auth::token::verify_jwt;
use axum::{
    body::Body,
    extract::State,
    http::{Request, StatusCode},
    middleware::Next,
    response::Response,
};
use sqlx::PgPool;

pub async fn auth_middleware(
    State(pool): State<PgPool>,
    mut req: Request<Body>,
    next: Next,
) -> Result<Response, StatusCode> {
    tracing::info!("AUTH middleware entered");
    let auth_header = req
        .headers()
        .get("Authorization")
        .and_then(|h| h.to_str().ok())
        .ok_or_else(|| {
            tracing::error!("Missing Authorization header");
            StatusCode::UNAUTHORIZED
        })?;

    tracing::info!("Authorization header OK");
    let token = auth_header.strip_prefix("Bearer ").ok_or_else(|| {
        tracing::error!("Missing Bearer prefix");
        StatusCode::UNAUTHORIZED
    })?;

    tracing::info!("Bearer token extracted");
    let secret = std::env::var("JWT_SECRET").unwrap_or_else(|_| "STAR_WARS_ROCKS".to_string());

    let claims = verify_jwt(token, &secret).await.map_err(|e| {
        tracing::error!("JWT verification failed: {:?}", e);
        StatusCode::UNAUTHORIZED
    })?;

    tracing::info!("JWT valid for user: {}", claims.user);

    let token_valid: Option<(String,)> = sqlx::query_as(
        "SELECT token FROM user_login
         WHERE username = $1 AND token = $2 AND expire_datetime > NOW()",
    )
    .bind(&claims.user)
    .bind(token)
    .fetch_optional(&pool)
    .await
    .map_err(|e| {
        tracing::error!("DB error: {:?}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    if token_valid.is_none() {
        tracing::error!("Token not found or expired in DB");
        return Err(StatusCode::UNAUTHORIZED);
    }

    tracing::info!("Token validated against DB");

    req.extensions_mut().insert(claims.user);

    Ok(next.run(req).await)
}
