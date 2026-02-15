use crate::core::error::AppError;
use chrono::{Duration, Utc};
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::domain::users::entity::UserRole;

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: Uuid,
    pub role: UserRole,
    pub token_type: TokenType,
    pub exp: i64,
    pub iat: i64,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub enum TokenType {
    Access,
    Refresh,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TokenPair {
    pub access_token: String,
    pub refresh_token: String,
}

pub fn generate_token_pair(
    user_id: Uuid,
    role: UserRole,
    secret: &str,
) -> Result<TokenPair, AppError> {
    let access_token = generate_token(
        user_id,
        role.clone(),
        secret,
        TokenType::Access,
        Duration::days(1),
    )?;
    let refresh_token =
        generate_token(user_id, role, secret, TokenType::Refresh, Duration::days(7))?;

    Ok(TokenPair {
        access_token,
        refresh_token,
    })
}

fn generate_token(
    user_id: Uuid,
    role: UserRole,
    secret: &str,
    token_type: TokenType,
    expires_in: Duration,
) -> Result<String, AppError> {
    let now = Utc::now();
    let expire = now + expires_in;

    let claims = Claims {
        sub: user_id,
        role,
        token_type,
        exp: expire.timestamp(),
        iat: now.timestamp(),
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_ref()),
    )
    .map_err(|e| AppError::Database(format!("Token generation error: {}", e)))
}

pub fn verify_token(token: &str, secret: &str) -> Result<Claims, AppError> {
    decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_ref()),
        &Validation::default(),
    )
    .map(|data| data.claims)
    .map_err(|_| AppError::Unauthorized("Invalid token".to_string()))
}
