use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use validator::Validate;
use uuid::Uuid;
use crate::shared::dto::pagination::PaginationQuery;

#[derive(Debug, Deserialize, Serialize, Clone, Validate, ToSchema, Default)]
pub struct GetProductsQuery {
    #[serde(flatten)]
    pub pagination: PaginationQuery,

    pub category_id: Option<Uuid>,

    pub material_id: Option<Uuid>,

    pub foundation_id: Option<Uuid>,
}
