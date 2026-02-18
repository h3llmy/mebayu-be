use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use validator::Validate;

#[derive(Debug, Serialize, Deserialize, Validate, ToSchema)]
pub struct LoginDto {
    #[validate(length(min = 3))]
    pub username: String,

    #[validate(length(min = 6))]
    pub password: String,
}
