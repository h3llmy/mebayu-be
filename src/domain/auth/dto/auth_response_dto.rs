use crate::domain::users::dto::user_response_dto::UserResponseDto;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct AuthResponseDto {
    pub user: UserResponseDto,
    pub access_token: String,
    pub refresh_token: String,
}
