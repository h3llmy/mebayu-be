use crate::core::error::AppError;
use bcrypt::{DEFAULT_COST, hash, verify};

pub fn hash_password(password: &str) -> Result<String, AppError> {
    hash(password, DEFAULT_COST).map_err(|e| AppError::Internal(format!("Hashing error: {}", e)))
}

pub fn verify_password(password: &str, hashed: &str) -> Result<bool, AppError> {
    verify(password, hashed).map_err(|e| AppError::Internal(format!("Verification error: {}", e)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_hash_and_verify_success() {
        let password = "super_secret_password_123";

        // 1. Test Hashing
        let hashed_result = hash_password(password);
        assert!(hashed_result.is_ok(), "Password hashing should succeed");

        let hashed = hashed_result.unwrap();
        assert_ne!(
            password, hashed,
            "Hashed password should not match plaintext"
        );
        assert!(
            hashed.starts_with("$2b$") || hashed.starts_with("$2a$") || hashed.starts_with("$2y$"),
            "Hash should have standard bcrypt prefix"
        );

        // 2. Test Verification with the correct password
        let verify_result = verify_password(password, &hashed);
        assert!(
            verify_result.is_ok(),
            "Verification should process without throwing an AppError"
        );
        assert!(
            verify_result.unwrap(),
            "Verification should return true for the correct password"
        );
    }

    #[tokio::test]
    async fn test_verify_wrong_password() {
        let password = "super_secret_password_123";
        let wrong_password = "wrong_password_456";

        let hashed = hash_password(password).expect("Failed to hash password");

        // Verify with the wrong password
        let verify_result = verify_password(wrong_password, &hashed);

        // It shouldn't error out (Err), it should successfully process and return `false`
        assert!(
            verify_result.is_ok(),
            "Verification should not throw an error for a valid hash"
        );
        assert!(
            !verify_result.unwrap(),
            "Verification should return false for the wrong password"
        );
    }

    #[tokio::test]
    async fn test_verify_malformed_hash() {
        let password = "some_password";
        let malformed_hash = "this_is_not_a_valid_bcrypt_hash_string";

        // Attempting to verify against a completely invalid hash structure should trigger `bcrypt::BcryptError`
        let verify_result = verify_password(password, malformed_hash);

        assert!(
            verify_result.is_err(),
            "Verification should fail with an error for a malformed hash"
        );

        // Ensure it maps to your AppError::Internal variant
        assert!(matches!(verify_result.unwrap_err(), AppError::Internal(_)));
    }
}
