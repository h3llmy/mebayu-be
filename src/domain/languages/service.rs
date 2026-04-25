use std::sync::Arc;
use uuid::Uuid;
use chrono::Utc;

use crate::{
    core::error::AppError,
    domain::languages::{
        entity::Language,
        repository::LanguageRepository,
        dto::{CreateLanguageRequest, UpdateLanguageRequest},
    },
};

pub struct LanguageService {
    repository: Arc<dyn LanguageRepository>,
}

impl LanguageService {
    pub fn new(repository: Arc<dyn LanguageRepository>) -> Self {
        Self { repository }
    }

    pub async fn get_all(&self) -> Result<Vec<Language>, AppError> {
        self.repository.find_all().await
    }

    pub async fn get_by_id(&self, id: Uuid) -> Result<Language, AppError> {
        self.repository.find_by_id(id).await
    }

    pub async fn create(&self, req: CreateLanguageRequest) -> Result<Language, AppError> {
        let language = Language {
            id: Uuid::new_v4(),
            code: req.code,
            name: req.name,
            is_default: req.is_default,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        self.repository.create(&language).await
    }

    pub async fn update(&self, id: Uuid, req: UpdateLanguageRequest) -> Result<Language, AppError> {
        let existing = self.repository.find_by_id(id).await?;
        
        let language = Language {
            id,
            code: req.code.unwrap_or(existing.code),
            name: req.name.unwrap_or(existing.name),
            is_default: req.is_default.unwrap_or(existing.is_default),
            created_at: existing.created_at,
            updated_at: Utc::now(),
        };

        self.repository.update(id, &language).await
    }

    pub async fn delete(&self, id: Uuid) -> Result<(), AppError> {
        self.repository.delete(id).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::languages::repository::MockLanguageRepository;

    #[tokio::test]
    async fn test_get_all() {
        let mut mock_repo = MockLanguageRepository::new();
        let languages = vec![Language {
            id: Uuid::new_v4(),
            code: "en".to_string(),
            name: "English".to_string(),
            is_default: true,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }];

        let languages_clone = languages.clone();
        mock_repo
            .expect_find_all()
            .times(1)
            .returning(move || Ok(languages_clone.clone()));

        let service = LanguageService::new(Arc::new(mock_repo));
        let result = service.get_all().await.unwrap();

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].code, "en");
    }

    #[tokio::test]
    async fn test_get_by_id() {
        let mut mock_repo = MockLanguageRepository::new();
        let id = Uuid::new_v4();
        let language = Language {
            id,
            code: "en".to_string(),
            name: "English".to_string(),
            is_default: true,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let language_clone = language.clone();
        mock_repo
            .expect_find_by_id()
            .with(mockall::predicate::eq(id))
            .times(1)
            .returning(move |_| Ok(language_clone.clone()));

        let service = LanguageService::new(Arc::new(mock_repo));
        let result = service.get_by_id(id).await.unwrap();

        assert_eq!(result.id, id);
        assert_eq!(result.code, "en");
    }

    #[tokio::test]
    async fn test_create() {
        let mut mock_repo = MockLanguageRepository::new();
        let req = CreateLanguageRequest {
            code: "id".to_string(),
            name: "Indonesia".to_string(),
            is_default: false,
        };

        mock_repo
            .expect_create()
            .times(1)
            .returning(|lang| Ok(lang.clone()));

        let service = LanguageService::new(Arc::new(mock_repo));
        let result = service.create(req).await.unwrap();

        assert_eq!(result.code, "id");
        assert_eq!(result.name, "Indonesia");
    }

    #[tokio::test]
    async fn test_update() {
        let mut mock_repo = MockLanguageRepository::new();
        let id = Uuid::new_v4();
        let existing = Language {
            id,
            code: "en".to_string(),
            name: "English".to_string(),
            is_default: true,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        let req = UpdateLanguageRequest {
            code: Some("id".to_string()),
            name: None,
            is_default: Some(false),
        };

        let existing_clone = existing.clone();
        mock_repo
            .expect_find_by_id()
            .with(mockall::predicate::eq(id))
            .returning(move |_| Ok(existing_clone.clone()));

        mock_repo
            .expect_update()
            .with(mockall::predicate::eq(id), mockall::predicate::always())
            .times(1)
            .returning(|_, updated| Ok(updated.clone()));

        let service = LanguageService::new(Arc::new(mock_repo));
        let result = service.update(id, req).await.unwrap();

        assert_eq!(result.code, "id");
        assert_eq!(result.name, "English");
        assert_eq!(result.is_default, false);
    }

    #[tokio::test]
    async fn test_delete() {
        let mut mock_repo = MockLanguageRepository::new();
        let id = Uuid::new_v4();

        mock_repo
            .expect_delete()
            .with(mockall::predicate::eq(id))
            .times(1)
            .returning(|_| Ok(()));

        let service = LanguageService::new(Arc::new(mock_repo));
        let result = service.delete(id).await;

        assert!(result.is_ok());
    }
}
