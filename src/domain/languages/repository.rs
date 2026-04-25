use super::entity::Language;
use crate::core::error::AppError;
use async_trait::async_trait;
use uuid::Uuid;

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait LanguageRepository: Send + Sync {
    async fn find_all(&self) -> Result<Vec<Language>, AppError>;
    async fn find_by_code(&self, code: &str) -> Result<Language, AppError>;
    async fn find_by_id(&self, id: Uuid) -> Result<Language, AppError>;
    async fn create(&self, language: &Language) -> Result<Language, AppError>;
    async fn update(&self, id: Uuid, language: &Language) -> Result<Language, AppError>;
    async fn delete(&self, id: Uuid) -> Result<(), AppError>;
}
