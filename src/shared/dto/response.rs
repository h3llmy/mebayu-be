use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ApiResponse<T> {
    pub data: T,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PaginationResponse<T> {
    pub data: T,
    pub page: u32,
    pub limit: u32,
    pub total_data: u64,
    pub total_page: u64,
}
