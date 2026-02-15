use async_trait::async_trait;
use std::sync::Arc;
use uuid::Uuid;

use crate::{
    core::error::AppError,
    domain::product_categories::dto::{CreateProductCategoryRequest, UpdateProductCategoryRequest},
    shared::dto::pagination::PaginationQuery,
};

use super::entity::ProductCategory;

#[async_trait]
pub trait ProductCategoryRepository: Send + Sync {
    async fn find_all(
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

pub struct ProductCategoryServiceImpl<R: ProductCategoryRepository> {
    repository: Arc<R>,
}

impl<R: ProductCategoryRepository> ProductCategoryServiceImpl<R> {
    pub fn new(repository: Arc<R>) -> Self {
        Self { repository }
    }

    pub async fn get_all(
        &self,
        query: &PaginationQuery,
    ) -> Result<(Vec<ProductCategory>, u64), AppError> {
        self.repository.find_all(query).await
    }

    pub async fn get_by_id(&self, id: Uuid) -> Result<ProductCategory, AppError> {
        self.repository.find_by_id(id).await
    }

    pub async fn create(
        &self,
        req: CreateProductCategoryRequest,
    ) -> Result<ProductCategory, AppError> {
        let category = ProductCategory {
            id: Uuid::new_v4(),
            name: req.name.unwrap(),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        self.repository.create(&category).await
    }

    pub async fn update(
        &self,
        id: Uuid,
        req: UpdateProductCategoryRequest,
    ) -> Result<ProductCategory, AppError> {
        let category = self.repository.find_by_id(id).await?;
        let category = ProductCategory {
            id,
            name: req.name.unwrap_or(category.name),
            created_at: category.created_at,
            updated_at: chrono::Utc::now(),
        };
        self.repository.update(id, &category).await
    }

    pub async fn delete(&self, id: Uuid) -> Result<(), AppError> {
        self.repository.delete(id).await
    }
}
