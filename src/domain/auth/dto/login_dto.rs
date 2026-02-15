use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct LoginDto {
    #[validate(required(message = "Username is required"))]
    #[validate(length(min = 3, message = "Username must be at least 3 characters"))]
    pub username: Option<String>,

    #[validate(required(message = "Password is required"))]
    #[validate(length(min = 6, message = "Password must be at least 6 characters"))]
    pub password: Option<String>,
}
