use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Clone, Serialize, Deserialize, FromRow, ToSchema)]
pub struct Language {
    pub id: Uuid,
    pub code: String,
    pub name: String,
    pub is_default: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
