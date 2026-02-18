use async_trait::async_trait;
use std::sync::Arc;
use uuid::Uuid;

use crate::{
    core::error::AppError,
    domain::product_materials::dto::{CreateProductMaterialRequest, UpdateProductMaterialRequest},
    shared::dto::{pagination::PaginationQuery, response::PaginationResponse},
};

use super::entity::ProductMaterial;

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

pub struct ProductMaterialServiceImpl<R: ProductMaterialRepository> {
    repository: Arc<R>,
}

impl<R: ProductMaterialRepository> ProductMaterialServiceImpl<R> {
    pub fn new(repository: Arc<R>) -> Self {
        Self { repository }
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
        let material = ProductMaterial {
            id: Uuid::new_v4(),
            name: req.name,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        self.repository.create(&material).await
    }

    pub async fn update(
        &self,
        id: Uuid,
        req: UpdateProductMaterialRequest,
    ) -> Result<ProductMaterial, AppError> {
        let material = self.repository.find_by_id(id).await?;
        let material = ProductMaterial {
            id,
            name: req.name.unwrap_or(material.name),
            created_at: material.created_at,
            updated_at: chrono::Utc::now(),
        };
        self.repository.update(id, &material).await
    }

    pub async fn delete(&self, id: Uuid) -> Result<(), AppError> {
        self.repository.delete(id).await
    }
}
