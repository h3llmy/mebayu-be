use async_trait::async_trait;
use std::sync::Arc;
use uuid::Uuid;

use crate::{
    core::error::AppError,
    domain::product_foundations::dto::{CreateProductFoundationRequest, UpdateProductFoundationRequest},
    shared::dto::{pagination::PaginationQuery, response::PaginationResponse},
};

use super::entity::ProductFoundation;

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait ProductFoundationRepository: Send + Sync {
    async fn find_all(
        &self,
        query: &PaginationQuery,
    ) -> Result<(Vec<ProductFoundation>, u64), AppError>;
    async fn find_by_id(&self, id: Uuid) -> Result<ProductFoundation, AppError>;
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
}

impl ProductFoundationServiceImpl {
    pub fn new(repository: Arc<dyn ProductFoundationRepository>) -> Self {
        Self { repository }
    }

    pub async fn get_all(
        &self,
        query: &PaginationQuery,
    ) -> Result<PaginationResponse<Vec<ProductFoundation>>, AppError> {
        let (foundations, total_data) = self.repository.find_all(query).await?;
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

    pub async fn get_by_id(&self, id: Uuid) -> Result<ProductFoundation, AppError> {
        self.repository.find_by_id(id).await
    }

    pub async fn create(
        &self,
        req: CreateProductFoundationRequest,
    ) -> Result<ProductFoundation, AppError> {
        let foundation = ProductFoundation {
            id: Uuid::new_v4(),
            name: req.name,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        self.repository.create(&foundation).await
    }

    pub async fn update(
        &self,
        id: Uuid,
        req: UpdateProductFoundationRequest,
    ) -> Result<ProductFoundation, AppError> {
        let foundation = self.repository.find_by_id(id).await?;
        let foundation = ProductFoundation {
            id,
            name: req.name.unwrap_or(foundation.name),
            created_at: foundation.created_at,
            updated_at: chrono::Utc::now(),
        };
        self.repository.update(id, &foundation).await
    }

    pub async fn delete(&self, id: Uuid) -> Result<(), AppError> {
        self.repository.delete(id).await
    }
}
