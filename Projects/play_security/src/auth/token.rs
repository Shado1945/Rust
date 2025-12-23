use chrono::{Duration, Utc};
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};
use serde::{Deserialize, Serialize};
use tracing::error;

use crate::response::responses::Response;

#[derive(Debug, Serialize, Deserialize)]
pub struct Token {
    pub user: String,
    pub exp: usize,
    pub iat: usize,
}

pub async fn create_jwt(username: &str, secret: &str) -> Result<String, Response> {
    let now = Utc::now();
    let expires_at = now + Duration::hours(8);
    let claims = Token {
        user: username.to_owned(),
        iat: now.timestamp() as usize,
        exp: expires_at.timestamp() as usize,
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .map_err(|e| {
        error!("JWT Creation error: {:?}", e);
        Response::InternalError
    })
}

pub async fn verify_jwt(token: &str, secret: &str) -> Result<Token, Response> {
    let token_data = decode::<Token>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::default(),
    )?;
    Ok(token_data.claims)
}
