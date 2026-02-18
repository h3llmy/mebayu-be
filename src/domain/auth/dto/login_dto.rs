use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct LoginDto {
    #[validate(required)]
    #[validate(length(min = 3))]
    pub username: Option<String>,

    #[validate(required)]
    #[validate(length(min = 6))]
    pub password: Option<String>,
}
