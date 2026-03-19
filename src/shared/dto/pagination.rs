use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};
use validator::Validate;

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, ToSchema, Default)]
pub enum SortOrder {
    #[default]
    Asc,
    Desc,
}

#[derive(Debug, Deserialize, Serialize, Clone, Validate, ToSchema, IntoParams, Default)]
#[into_params(parameter_in = Query)]
pub struct PaginationQuery {
    #[serde(default, deserialize_with = "deserialize_option_number_from_string")]
    pub page: Option<u32>,

    #[serde(default, deserialize_with = "deserialize_option_number_from_string")]
    pub limit: Option<u32>,

    pub search: Option<String>,

    pub sort: Option<String>,

    #[param(inline)]
    pub sort_order: Option<SortOrder>,
}

pub fn deserialize_option_number_from_string<'de, T, D>(deserializer: D) -> Result<Option<T>, D::Error>
where
    T: std::str::FromStr,
    T::Err: std::fmt::Display,
    D: serde::Deserializer<'de>,
{
    use serde::Deserialize;
    let opt = Option::<String>::deserialize(deserializer)?;
    match opt {
        Some(s) if s.is_empty() => Ok(None),
        Some(s) => s.parse().map(Some).map_err(serde::de::Error::custom),
        None => Ok(None),
    }
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
