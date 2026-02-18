use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct UpdateUserDto {
    #[validate(length(min = 3))]
    pub username: Option<String>,

    #[validate(email)]
    pub email: Option<String>,

    #[validate(length(min = 8))]
    pub password: Option<String>,
}
