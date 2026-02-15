use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct RefreshTokenDto {
    #[validate(required(message = "Refresh token is required"))]
    pub refresh_token: Option<String>,
}
