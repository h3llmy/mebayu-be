use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct RegisterDto {
    #[validate(required(message = "Username is required"))]
    #[validate(length(min = 3, message = "Username must be at least 3 characters"))]
    pub username: Option<String>,

    #[validate(required(message = "Email is required"))]
    #[validate(email(message = "Invalid email format"))]
    pub email: Option<String>,

    #[validate(required(message = "Password is required"))]
    #[validate(length(min = 8, message = "Password must be at least 8 characters"))]
    pub password: Option<String>,
}
