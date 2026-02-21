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

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;
    use uuid::Uuid;

    const TEST_SECRET: &str = "my_super_secret_test_key_1234567890";

    // Helper function to provide a mock role for testing.
    // Note: Replace `UserRole::User` with an actual variant from your `UserRole` enum.
    fn get_mock_role() -> UserRole {
        UserRole::User
    }

    #[tokio::test]
    async fn test_generate_token_pair_success() {
        let user_id = Uuid::new_v4();
        let role = get_mock_role();

        // 1. Generate the pair
        let result = generate_token_pair(user_id, role.clone(), TEST_SECRET);
        assert!(result.is_ok(), "Token pair generation should succeed");

        let pair = result.unwrap();
        assert!(
            !pair.access_token.is_empty(),
            "Access token should not be empty"
        );
        assert!(
            !pair.refresh_token.is_empty(),
            "Refresh token should not be empty"
        );

        // 2. Verify Access Token
        let access_claims = verify_token(&pair.access_token, TEST_SECRET)
            .expect("Failed to verify valid access token");

        assert_eq!(access_claims.sub, user_id);
        assert_eq!(access_claims.token_type, TokenType::Access);
        // Note: Uncomment the next line if your `UserRole` derives `PartialEq`
        // assert_eq!(access_claims.role, role);

        // 3. Verify Refresh Token
        let refresh_claims = verify_token(&pair.refresh_token, TEST_SECRET)
            .expect("Failed to verify valid refresh token");

        assert_eq!(refresh_claims.sub, user_id);
        assert_eq!(refresh_claims.token_type, TokenType::Refresh);
    }

    #[tokio::test]
    async fn test_verify_token_invalid_secret() {
        let user_id = Uuid::new_v4();
        let pair = generate_token_pair(user_id, get_mock_role(), TEST_SECRET)
            .expect("Failed to generate tokens");

        let wrong_secret = "invalid_secret_key";

        // Attempting to verify with the wrong secret should fail
        let result = verify_token(&pair.access_token, wrong_secret);
        assert!(
            result.is_err(),
            "Verification should fail with an invalid secret"
        );

        // Ensure it maps to your specific unauthorized error
        // Note: This matches macro assumes your AppError has an Unauthorized variant
        // with a String as seen in your code.
        assert!(matches!(result.unwrap_err(), AppError::Unauthorized(_)));
    }

    #[tokio::test]
    async fn test_verify_token_malformed() {
        let malformed_token = "this.is.not_a_real_jwt_token";

        // Decoding nonsense should immediately fail
        let result = verify_token(malformed_token, TEST_SECRET);
        assert!(
            result.is_err(),
            "Verification should fail for malformed tokens"
        );
        assert!(matches!(result.unwrap_err(), AppError::Unauthorized(_)));
    }

    #[tokio::test]
    async fn test_token_expiration() {
        let user_id = Uuid::new_v4();

        // Generate a token with a negative duration so it is instantly expired
        let expired_token = generate_token(
            user_id,
            get_mock_role(),
            TEST_SECRET,
            TokenType::Access,
            Duration::days(-1),
        )
        .expect("Failed to generate expired token");

        // The default validation in `jsonwebtoken` checks the `exp` claim automatically
        let result = verify_token(&expired_token, TEST_SECRET);

        assert!(
            result.is_err(),
            "Verification should fail for an expired token"
        );
        assert!(matches!(result.unwrap_err(), AppError::Unauthorized(_)));
    }
}
