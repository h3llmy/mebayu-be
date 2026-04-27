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
