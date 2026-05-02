use async_trait::async_trait;
use std::sync::Arc;
use uuid::Uuid;

use crate::{
    core::error::AppError,
    domain::{
        languages::repository::LanguageRepository,
        product_foundations::{
            dto::{CreateProductFoundationRequest, UpdateProductFoundationRequest},
            entity::{ProductFoundation, ProductFoundationTranslation},
        },
    },
    shared::dto::{pagination::PaginationQuery, response::PaginationResponse},
};

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait ProductFoundationRepository: Send + Sync {
    async fn find_all(
        &self,
        query: &PaginationQuery,
        language_id: Option<Uuid>,
    ) -> Result<(Vec<ProductFoundation>, u64), AppError>;
    async fn find_by_id(&self, id: Uuid, language_id: Option<Uuid>) -> Result<ProductFoundation, AppError>;
    async fn create(&self, foundation: &ProductFoundation) -> Result<ProductFoundation, AppError>;
    async fn update(
        &self,
        id: Uuid,
        foundation: &ProductFoundation,
    ) -> Result<ProductFoundation, AppError>;
    async fn delete(&self, id: Uuid) -> Result<(), AppError>;
}

pub struct ProductFoundationServiceImpl {
    repository: Arc<dyn ProductFoundationRepository>,
    language_repository: Arc<dyn LanguageRepository>,
}

impl ProductFoundationServiceImpl {
    pub fn new(
        repository: Arc<dyn ProductFoundationRepository>,
        language_repository: Arc<dyn LanguageRepository>,
    ) -> Self {
        Self {
            repository,
            language_repository,
        }
    }

    pub async fn get_all(
        &self,
        query: &PaginationQuery,
        language_id: Option<Uuid>,
    ) -> Result<PaginationResponse<Vec<ProductFoundation>>, AppError> {
        let (foundations, total_data) = self.repository.find_all(query, language_id).await?;
        let limit = query.get_limit();
        let total_page = (total_data as f64 / limit as f64).ceil() as u64;

        Ok(PaginationResponse {
            data: foundations,
            page: query.get_page(),
            limit,
            total_data,
            total_page,
        })
    }

    pub async fn get_by_id(&self, id: Uuid, language_id: Option<Uuid>) -> Result<ProductFoundation, AppError> {
        self.repository.find_by_id(id, language_id).await
    }

    pub async fn create(
        &self,
        req: CreateProductFoundationRequest,
    ) -> Result<ProductFoundation, AppError> {
        let id = Uuid::new_v4();
        let mut translations = Vec::new();

        for t_req in req.translations {
            let language = self
                .language_repository
                .find_by_code(&t_req.language_code)
                .await?;
            translations.push(ProductFoundationTranslation {
                foundation_id: id, language: None,
                language_id: language.id,
                name: t_req.name,
            });
        }

        let foundation = ProductFoundation {
            id,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            translations,
        };

        self.repository.create(&foundation).await?;
        self.repository.find_by_id(id, None).await
    }

    pub async fn update(
        &self,
        id: Uuid,
        req: UpdateProductFoundationRequest,
    ) -> Result<ProductFoundation, AppError> {
        let existing = self.repository.find_by_id(id, None).await?;
        let mut translations = Vec::new();

        if let Some(req_translations) = req.translations {
            for t_req in req_translations {
                let language = self
                    .language_repository
                    .find_by_code(&t_req.language_code)
                    .await?;
                translations.push(ProductFoundationTranslation {
                    foundation_id: id, language: None,
                    language_id: language.id,
                    name: t_req.name,
                });
            }
        } else {
            translations = existing.translations;
        }

        let foundation = ProductFoundation {
            id,
            created_at: existing.created_at,
            updated_at: chrono::Utc::now(),
            translations,
        };
        self.repository.update(id, &foundation).await?;
        self.repository.find_by_id(id, None).await
    }

    pub async fn delete(&self, id: Uuid) -> Result<(), AppError> {
        self.repository.delete(id).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{
        languages::{entity::Language, repository::MockLanguageRepository},
        product_foundations::{
            dto::{CreateProductFoundationRequest, ProductFoundationTranslationRequest, UpdateProductFoundationRequest},
            entity::{ProductFoundation, ProductFoundationTranslation},
        },
    };
    use chrono::Utc;

    fn test_lang_id() -> Uuid { Uuid::new_v4() }

    fn make_foundation(id: Uuid, lang_id: Uuid, name: &str) -> ProductFoundation {
        ProductFoundation {
            id,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            translations: vec![ProductFoundationTranslation {
                foundation_id: id,
                language_id: lang_id,
                language: None,
                name: name.to_string(),
            }],
        }
    }

    #[tokio::test]
    async fn test_get_by_id() {
        let mut mock_repo = MockProductFoundationRepository::new();
        let mock_lang_repo = MockLanguageRepository::new();
        let id = Uuid::new_v4();
        let lang_id = test_lang_id();
        let expected = make_foundation(id, lang_id, "Hardwood");

        let expected_clone = expected.clone();
        mock_repo
            .expect_find_by_id()
            .with(mockall::predicate::eq(id), mockall::predicate::always())
            .times(1)
            .returning(move |_, _| Ok(expected_clone.clone()));

        let service = ProductFoundationServiceImpl::new(Arc::new(mock_repo), Arc::new(mock_lang_repo));
        let result = service.get_by_id(id, None).await.unwrap();

        assert_eq!(result.id, id);
        assert_eq!(result.translations[0].name, "Hardwood");
    }

    #[tokio::test]
    async fn test_get_all() {
        let mut mock_repo = MockProductFoundationRepository::new();
        let mock_lang_repo = MockLanguageRepository::new();
        let id = Uuid::new_v4();
        let lang_id = test_lang_id();
        let query = crate::shared::dto::pagination::PaginationQuery::default();
        let foundations = vec![make_foundation(id, lang_id, "Marble")];
        let total_data = 1u64;

        let foundations_clone = foundations.clone();
        mock_repo
            .expect_find_all()
            .times(1)
            .returning(move |_, _| Ok((foundations_clone.clone(), total_data)));

        let service = ProductFoundationServiceImpl::new(Arc::new(mock_repo), Arc::new(mock_lang_repo));
        let result = service.get_all(&query, None).await.unwrap();

        assert_eq!(result.total_data, total_data);
        assert_eq!(result.data.len(), 1);
        assert_eq!(result.data[0].translations[0].name, "Marble");
    }

    #[tokio::test]
    async fn test_create() {
        let mut mock_repo = MockProductFoundationRepository::new();
        let mut mock_lang_repo = MockLanguageRepository::new();
        let lang_id = test_lang_id();

        let req = CreateProductFoundationRequest {
            translations: vec![ProductFoundationTranslationRequest {
                language_code: "en".to_string(),
                name: "Granite".to_string(),
            }],
        };

        mock_lang_repo
            .expect_find_by_code()
            .with(mockall::predicate::eq("en"))
            .returning(move |code| Ok(Language {
                id: lang_id,
                code: code.to_string(),
                name: "English".to_string(),
                is_default: true,
                created_at: Utc::now(),
                updated_at: Utc::now(),
            }));

        mock_repo
            .expect_create()
            .times(1)
            .returning(|f| Ok(f.clone()));

        mock_repo
            .expect_find_by_id()
            .returning(move |id, _| Ok(ProductFoundation {
                id,
                created_at: Utc::now(),
                updated_at: Utc::now(),
                translations: vec![ProductFoundationTranslation {
                    foundation_id: id,
                    language_id: lang_id,
                    language: None,
                    name: "Granite".to_string(),
                }],
            }));

        let service = ProductFoundationServiceImpl::new(Arc::new(mock_repo), Arc::new(mock_lang_repo));
        let result = service.create(req).await.unwrap();

        assert_eq!(result.translations[0].name, "Granite");
        assert_eq!(result.translations[0].language_id, lang_id);
    }

    #[tokio::test]
    async fn test_update_with_new_translations() {
        let mut mock_repo = MockProductFoundationRepository::new();
        let mut mock_lang_repo = MockLanguageRepository::new();
        let id = Uuid::new_v4();
        let lang_id = test_lang_id();
        let existing = make_foundation(id, lang_id, "Old Name");

        let req = UpdateProductFoundationRequest {
            translations: Some(vec![ProductFoundationTranslationRequest {
                language_code: "en".to_string(),
                name: "New Name".to_string(),
            }]),
        };

        let existing_clone = existing.clone();
        let toggle = std::sync::Arc::new(std::sync::Mutex::new(false));
        mock_repo
            .expect_find_by_id()
            .with(mockall::predicate::eq(id), mockall::predicate::always())
            .returning(move |_, _| {
                let mut t = toggle.lock().unwrap();
                if !*t {
                    *t = true;
                    Ok(existing_clone.clone())
                } else {
                    let mut updated = existing_clone.clone();
                    if let Some(tr) = updated.translations.first_mut() {
                        tr.name = "New Name".to_string();
                    }
                    Ok(updated)
                }
            });

        mock_lang_repo
            .expect_find_by_code()
            .with(mockall::predicate::eq("en"))
            .returning(move |code| Ok(Language {
                id: lang_id,
                code: code.to_string(),
                name: "English".to_string(),
                is_default: true,
                created_at: Utc::now(),
                updated_at: Utc::now(),
            }));

        mock_repo
            .expect_update()
            .with(mockall::predicate::eq(id), mockall::predicate::always())
            .times(1)
            .returning(|_, f| Ok(f.clone()));

        let service = ProductFoundationServiceImpl::new(Arc::new(mock_repo), Arc::new(mock_lang_repo));
        let result = service.update(id, req).await.unwrap();

        assert_eq!(result.translations[0].name, "New Name");
    }

    #[tokio::test]
    async fn test_update_preserves_translations_when_none() {
        let mut mock_repo = MockProductFoundationRepository::new();
        let mock_lang_repo = MockLanguageRepository::new();
        let id = Uuid::new_v4();
        let lang_id = test_lang_id();
        let existing = make_foundation(id, lang_id, "Keep Me");
        let existing_clone = existing.clone();

        mock_repo
            .expect_find_by_id()
            .with(mockall::predicate::eq(id), mockall::predicate::always())
            .returning(move |_, _| Ok(existing_clone.clone()));

        mock_repo
            .expect_update()
            .with(mockall::predicate::eq(id), mockall::predicate::always())
            .times(1)
            .returning(|_, f| Ok(f.clone()));

        let req = UpdateProductFoundationRequest { translations: None };

        let service = ProductFoundationServiceImpl::new(Arc::new(mock_repo), Arc::new(mock_lang_repo));
        let result = service.update(id, req).await.unwrap();

        assert_eq!(result.translations[0].name, "Keep Me");
    }

    #[tokio::test]
    async fn test_delete() {
        let mut mock_repo = MockProductFoundationRepository::new();
        let mock_lang_repo = MockLanguageRepository::new();
        let id = Uuid::new_v4();

        mock_repo
            .expect_delete()
            .with(mockall::predicate::eq(id))
            .times(1)
            .returning(|_| Ok(()));

        let service = ProductFoundationServiceImpl::new(Arc::new(mock_repo), Arc::new(mock_lang_repo));
        assert!(service.delete(id).await.is_ok());
    }

    #[tokio::test]
    async fn test_delete_propagates_error() {
        let mut mock_repo = MockProductFoundationRepository::new();
        let mock_lang_repo = MockLanguageRepository::new();

        mock_repo
            .expect_delete()
            .times(1)
            .returning(|_| Err(AppError::NotFound("Foundation not found".to_string())));

        let service = ProductFoundationServiceImpl::new(Arc::new(mock_repo), Arc::new(mock_lang_repo));
        let result = service.delete(Uuid::new_v4()).await;
        assert!(matches!(result, Err(AppError::NotFound(_))));
    }
}
