use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};
use validator::Validate;

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, ToSchema)]
pub enum SortOrder {
    Asc,
    Desc,
}

#[derive(Debug, Deserialize, Serialize, Clone, Validate, ToSchema, IntoParams)]
pub struct PaginationQuery {
    #[validate(range(min = 1))]
    pub page: Option<u32>,
    #[validate(range(min = 1))]
    pub limit: Option<u32>,
    pub search: Option<String>,
    pub sort: Option<String>,
    pub sort_order: Option<SortOrder>,
}

impl PaginationQuery {
    pub fn get_page(&self) -> u32 {
        self.page.unwrap_or(1)
    }

    pub fn get_limit(&self) -> u32 {
        self.limit.unwrap_or(10)
    }

    pub fn get_offset(&self) -> i64 {
        ((self.get_page() - 1) * self.get_limit()) as i64
    }

    pub fn get_search(&self) -> Option<String> {
        self.search.clone()
    }

    pub fn get_sort(&self) -> Option<String> {
        self.sort.clone()
    }

    pub fn get_sort_order(&self) -> Option<SortOrder> {
        self.sort_order.clone()
    }
}
