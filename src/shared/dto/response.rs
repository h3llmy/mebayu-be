use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
pub struct ApiResponse<T> {
    pub data: T,
}

#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
pub struct PaginationResponse<T> {
    pub data: T,
    pub page: u32,
    pub limit: u32,
    pub total_data: u64,
    pub total_page: u64,
}
