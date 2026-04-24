use async_trait::async_trait;
use uuid::Uuid;
use crate::core::error::AppError;
use super::entity::Language;

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait LanguageRepository: Send + Sync {
    async fn find_all(&self) -> Result<Vec<Language>, AppError>;
    async fn find_by_code(&self, code: &str) -> Result<Language, AppError>;
    async fn find_by_id(&self, id: Uuid) -> Result<Language, AppError>;
}
