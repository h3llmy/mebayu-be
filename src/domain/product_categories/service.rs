use async_trait::async_trait;
use std::sync::Arc;
use uuid::Uuid;

use crate::{
    core::error::AppError,
    domain::{
        languages::repository::LanguageRepository,
        product_categories::{
            dto::{CreateProductCategoryRequest, UpdateProductCategoryRequest},
            entity::{ProductCategory, ProductCategoryTranslation},
        },
    },
    shared::dto::{pagination::PaginationQuery, response::PaginationResponse},
};

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait ProductCategoryRepository: Send + Sync {
    async fn find_all(
        &self,
        query: &PaginationQuery,
    ) -> Result<(Vec<ProductCategory>, u64), AppError>;
    async fn find_all_with_product_count(
        &self,
        query: &PaginationQuery,
    ) -> Result<(Vec<ProductCategory>, u64), AppError>;
    async fn find_by_id(&self, id: Uuid) -> Result<ProductCategory, AppError>;
    async fn create(&self, category: &ProductCategory) -> Result<ProductCategory, AppError>;
    async fn update(
        &self,
        id: Uuid,
        category: &ProductCategory,
    ) -> Result<ProductCategory, AppError>;
    async fn delete(&self, id: Uuid) -> Result<(), AppError>;
}

pub struct ProductCategoryServiceImpl {
    repository: Arc<dyn ProductCategoryRepository>,
    language_repository: Arc<dyn LanguageRepository>,
}

impl ProductCategoryServiceImpl {
    pub fn new(
        repository: Arc<dyn ProductCategoryRepository>,
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
    ) -> Result<PaginationResponse<Vec<ProductCategory>>, AppError> {
        let (categories, total_data) = self.repository.find_all(query).await?;
        let limit = query.get_limit();
        let total_page = (total_data as f64 / limit as f64).ceil() as u64;

        Ok(PaginationResponse {
            data: categories,
            page: query.get_page(),
            limit,
            total_data,
            total_page,
        })
    }

    pub async fn get_all_with_product_count(
        &self,
        query: &PaginationQuery,
    ) -> Result<PaginationResponse<Vec<ProductCategory>>, AppError> {
        let (categories, total_data) = self.repository.find_all_with_product_count(query).await?;
        let limit = query.get_limit();
        let total_page = (total_data as f64 / limit as f64).ceil() as u64;

        Ok(PaginationResponse {
            data: categories,
            page: query.get_page(),
            limit,
            total_data,
            total_page,
        })
    }

    pub async fn get_by_id(&self, id: Uuid) -> Result<ProductCategory, AppError> {
        self.repository.find_by_id(id).await
    }

    pub async fn create(
        &self,
        req: CreateProductCategoryRequest,
    ) -> Result<ProductCategory, AppError> {
        let id = Uuid::new_v4();
        let mut translations = Vec::new();

        for t_req in req.translations {
            let language = self
                .language_repository
                .find_by_code(&t_req.language_code)
                .await?;
            translations.push(ProductCategoryTranslation {
                category_id: id,
                language_id: language.id,
                name: t_req.name,
            });
        }

        let category = ProductCategory {
            id,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            translations,
        };

        self.repository.create(&category).await
    }

    pub async fn update(
        &self,
        id: Uuid,
        req: UpdateProductCategoryRequest,
    ) -> Result<ProductCategory, AppError> {
        let existing = self.repository.find_by_id(id).await?;
        let mut translations = Vec::new();

        if let Some(req_translations) = req.translations {
            for t_req in req_translations {
                let language = self
                    .language_repository
                    .find_by_code(&t_req.language_code)
                    .await?;
                translations.push(ProductCategoryTranslation {
                    category_id: id,
                    language_id: language.id,
                    name: t_req.name,
                });
            }
        } else {
            translations = existing.translations;
        }

        let category = ProductCategory {
            id,
            created_at: existing.created_at,
            updated_at: chrono::Utc::now(),
            translations,
        };
        self.repository.update(id, &category).await
    }

    pub async fn delete(&self, id: Uuid) -> Result<(), AppError> {
        self.repository.delete(id).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::product_categories::entity::ProductCategory;
    use crate::domain::languages::entity::Language;
    use crate::domain::languages::repository::MockLanguageRepository;
    use chrono::Utc;

    fn test_lang_id() -> Uuid { Uuid::new_v4() }

    #[tokio::test]
    async fn test_get_by_id() {
        let mut mock_repo = MockProductCategoryRepository::new();
        let mock_lang_repo = MockLanguageRepository::new();
        let id = Uuid::new_v4();
        let expected_category = ProductCategory {
            id,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            translations: vec![ProductCategoryTranslation {
                category_id: id,
                language_id: test_lang_id(),
                name: "Test Category".to_string(),
            }],
        };

        let category_clone = expected_category.clone();
        mock_repo
            .expect_find_by_id()
            .with(mockall::predicate::eq(id))
            .times(1)
            .returning(move |_| Ok(category_clone.clone()));

        let service = ProductCategoryServiceImpl::new(Arc::new(mock_repo), Arc::new(mock_lang_repo));
        let result = service.get_by_id(id).await.unwrap();

        assert_eq!(result.id, expected_category.id);
        assert_eq!(result.translations[0].name, "Test Category");
    }

    #[tokio::test]
    async fn test_get_all() {
        let mut mock_repo = MockProductCategoryRepository::new();
        let mock_lang_repo = MockLanguageRepository::new();
        let query = PaginationQuery::default();
        let total_data = 1;
        let id = Uuid::new_v4();
        let categories = vec![ProductCategory {
            id,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            translations: vec![ProductCategoryTranslation {
                category_id: id,
                language_id: test_lang_id(),
                name: "Test".to_string(),
            }],
        }];

        let categories_clone = categories.clone();
        mock_repo
            .expect_find_all()
            .times(1)
            .returning(move |_| Ok((categories_clone.clone(), total_data)));

        let service = ProductCategoryServiceImpl::new(Arc::new(mock_repo), Arc::new(mock_lang_repo));
        let result = service.get_all(&query).await.unwrap();

        assert_eq!(result.total_data, total_data);
        assert_eq!(result.data.len(), 1);
    }

    #[tokio::test]
    async fn test_create() {
        let mut mock_repo = MockProductCategoryRepository::new();
        let mut mock_lang_repo = MockLanguageRepository::new();
        let lang_id = test_lang_id();
        let req = CreateProductCategoryRequest {
            translations: vec![ProductCategoryTranslationRequest {
                language_code: "en".to_string(),
                name: "New Category".to_string(),
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
            .returning(|category| Ok(category.clone()));

        let service = ProductCategoryServiceImpl::new(Arc::new(mock_repo), Arc::new(mock_lang_repo));
        let result = service.create(req).await.unwrap();

        assert_eq!(result.translations[0].name, "New Category");
    }

    #[tokio::test]
    async fn test_update() {
        let mut mock_repo = MockProductCategoryRepository::new();
        let mut mock_lang_repo = MockLanguageRepository::new();
        let id = Uuid::new_v4();
        let lang_id = test_lang_id();
        let existing = ProductCategory {
            id,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            translations: vec![ProductCategoryTranslation {
                category_id: id,
                language_id: lang_id,
                name: "Old Name".to_string(),
            }],
        };
        let req = UpdateProductCategoryRequest {
            translations: Some(vec![ProductCategoryTranslationRequest {
                language_code: "en".to_string(),
                name: "New Name".to_string(),
            }]),
        };

        let existing_clone = existing.clone();
        mock_repo
            .expect_find_by_id()
            .with(mockall::predicate::eq(id))
            .returning(move |_| Ok(existing_clone.clone()));

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
            .returning(|_, updated| Ok(updated.clone()));

        let service = ProductCategoryServiceImpl::new(Arc::new(mock_repo), Arc::new(mock_lang_repo));
        let result = service.update(id, req).await.unwrap();

        assert_eq!(result.translations[0].name, "New Name");
    }

    #[tokio::test]
    async fn test_delete() {
        let mut mock_repo = MockProductCategoryRepository::new();
        let mock_lang_repo = MockLanguageRepository::new();
        let id = Uuid::new_v4();

        mock_repo
            .expect_delete()
            .with(mockall::predicate::eq(id))
            .times(1)
            .returning(|_| Ok(()));

        let service = ProductCategoryServiceImpl::new(Arc::new(mock_repo), Arc::new(mock_lang_repo));
        let result = service.delete(id).await;

        assert!(result.is_ok());
    }
}
