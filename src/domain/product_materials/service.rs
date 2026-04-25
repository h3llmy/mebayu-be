use async_trait::async_trait;
use std::sync::Arc;
use uuid::Uuid;

use crate::{
    core::error::AppError,
    domain::{
        languages::repository::LanguageRepository,
        product_materials::{
            dto::{CreateProductMaterialRequest, UpdateProductMaterialRequest},
            entity::{ProductMaterial, ProductMaterialTranslation},
        },
    },
    shared::dto::{pagination::PaginationQuery, response::PaginationResponse},
};

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait ProductMaterialRepository: Send + Sync {
    async fn find_all(
        &self,
        query: &PaginationQuery,
    ) -> Result<(Vec<ProductMaterial>, u64), AppError>;
    async fn find_by_id(&self, id: Uuid) -> Result<ProductMaterial, AppError>;
    async fn create(&self, material: &ProductMaterial) -> Result<ProductMaterial, AppError>;
    async fn update(
        &self,
        id: Uuid,
        material: &ProductMaterial,
    ) -> Result<ProductMaterial, AppError>;
    async fn delete(&self, id: Uuid) -> Result<(), AppError>;
}

pub struct ProductMaterialServiceImpl {
    repository: Arc<dyn ProductMaterialRepository>,
    language_repository: Arc<dyn LanguageRepository>,
}

impl ProductMaterialServiceImpl {
    pub fn new(
        repository: Arc<dyn ProductMaterialRepository>,
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
    ) -> Result<PaginationResponse<Vec<ProductMaterial>>, AppError> {
        let (materials, total_data) = self.repository.find_all(query).await?;
        let limit = query.get_limit();
        let total_page = (total_data as f64 / limit as f64).ceil() as u64;

        Ok(PaginationResponse {
            data: materials,
            page: query.get_page(),
            limit,
            total_data,
            total_page,
        })
    }

    pub async fn get_by_id(&self, id: Uuid) -> Result<ProductMaterial, AppError> {
        self.repository.find_by_id(id).await
    }

    pub async fn create(
        &self,
        req: CreateProductMaterialRequest,
    ) -> Result<ProductMaterial, AppError> {
        let id = Uuid::new_v4();
        let mut translations = Vec::new();

        for t_req in req.translations {
            let language = self
                .language_repository
                .find_by_code(&t_req.language_code)
                .await?;
            translations.push(ProductMaterialTranslation {
                material_id: id,
                language_id: language.id,
                name: t_req.name,
            });
        }

        let material = ProductMaterial {
            id,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            translations,
        };

        self.repository.create(&material).await
    }

    pub async fn update(
        &self,
        id: Uuid,
        req: UpdateProductMaterialRequest,
    ) -> Result<ProductMaterial, AppError> {
        let existing = self.repository.find_by_id(id).await?;
        let mut translations = Vec::new();

        if let Some(req_translations) = req.translations {
            for t_req in req_translations {
                let language = self
                    .language_repository
                    .find_by_code(&t_req.language_code)
                    .await?;
                translations.push(ProductMaterialTranslation {
                    material_id: id,
                    language_id: language.id,
                    name: t_req.name,
                });
            }
        } else {
            translations = existing.translations;
        }

        let material = ProductMaterial {
            id,
            created_at: existing.created_at,
            updated_at: chrono::Utc::now(),
            translations,
        };
        self.repository.update(id, &material).await
    }

    pub async fn delete(&self, id: Uuid) -> Result<(), AppError> {
        self.repository.delete(id).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::languages::entity::Language;
    use crate::domain::languages::repository::MockLanguageRepository;
    use crate::domain::product_materials::entity::ProductMaterial;
    use crate::domain::product_materials::dto::ProductMaterialTranslationRequest;
    use chrono::Utc;

    fn test_lang_id() -> Uuid { Uuid::new_v4() }

    #[tokio::test]
    async fn test_get_by_id() {
        let mut mock_repo = MockProductMaterialRepository::new();
        let mock_lang_repo = MockLanguageRepository::new();
        let id = Uuid::new_v4();
        let expected_material = ProductMaterial {
            id,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            translations: vec![ProductMaterialTranslation {
                material_id: id,
                language_id: test_lang_id(),
                name: "Test Material".to_string(),
            }],
        };

        let material_clone = expected_material.clone();
        mock_repo
            .expect_find_by_id()
            .with(mockall::predicate::eq(id))
            .times(1)
            .returning(move |_| Ok(material_clone.clone()));

        let service = ProductMaterialServiceImpl::new(Arc::new(mock_repo), Arc::new(mock_lang_repo));
        let result = service.get_by_id(id).await.unwrap();

        assert_eq!(result.id, expected_material.id);
        assert_eq!(result.translations[0].name, "Test Material");
    }

    #[tokio::test]
    async fn test_get_all() {
        let mut mock_repo = MockProductMaterialRepository::new();
        let mock_lang_repo = MockLanguageRepository::new();
        let query = PaginationQuery::default();
        let total_data = 1;
        let id = Uuid::new_v4();
        let materials = vec![ProductMaterial {
            id,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            translations: vec![ProductMaterialTranslation {
                material_id: id,
                language_id: test_lang_id(),
                name: "Test".to_string(),
            }],
        }];

        let materials_clone = materials.clone();
        mock_repo
            .expect_find_all()
            .times(1)
            .returning(move |_| Ok((materials_clone.clone(), total_data)));

        let service = ProductMaterialServiceImpl::new(Arc::new(mock_repo), Arc::new(mock_lang_repo));
        let result = service.get_all(&query).await.unwrap();

        assert_eq!(result.total_data, total_data);
        assert_eq!(result.data.len(), 1);
    }

    #[tokio::test]
    async fn test_create() {
        let mut mock_repo = MockProductMaterialRepository::new();
        let mut mock_lang_repo = MockLanguageRepository::new();
        let lang_id = test_lang_id();
        let req = CreateProductMaterialRequest {
            translations: vec![ProductMaterialTranslationRequest {
                language_code: "en".to_string(),
                name: "New Material".to_string(),
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
            .returning(|material| Ok(material.clone()));

        let service = ProductMaterialServiceImpl::new(Arc::new(mock_repo), Arc::new(mock_lang_repo));
        let result = service.create(req).await.unwrap();

        assert_eq!(result.translations[0].name, "New Material");
    }

    #[tokio::test]
    async fn test_update() {
        let mut mock_repo = MockProductMaterialRepository::new();
        let mut mock_lang_repo = MockLanguageRepository::new();
        let id = Uuid::new_v4();
        let lang_id = test_lang_id();
        let existing = ProductMaterial {
            id,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            translations: vec![ProductMaterialTranslation {
                material_id: id,
                language_id: lang_id,
                name: "Old Name".to_string(),
            }],
        };
        let req = UpdateProductMaterialRequest {
            translations: Some(vec![ProductMaterialTranslationRequest {
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

        let service = ProductMaterialServiceImpl::new(Arc::new(mock_repo), Arc::new(mock_lang_repo));
        let result = service.update(id, req).await.unwrap();

        assert_eq!(result.translations[0].name, "New Name");
    }

    #[tokio::test]
    async fn test_delete() {
        let mut mock_repo = MockProductMaterialRepository::new();
        let mock_lang_repo = MockLanguageRepository::new();
        let id = Uuid::new_v4();

        mock_repo
            .expect_delete()
            .with(mockall::predicate::eq(id))
            .times(1)
            .returning(|_| Ok(()));

        let service = ProductMaterialServiceImpl::new(Arc::new(mock_repo), Arc::new(mock_lang_repo));
        let result = service.delete(id).await;

        assert!(result.is_ok());
    }
}
