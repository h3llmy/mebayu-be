use async_trait::async_trait;
use std::sync::Arc;
use uuid::Uuid;

use crate::{
    core::error::AppError,
    domain::products::dto::{CreateProductRequest, UpdateProductRequest},
    shared::dto::pagination::PaginationQuery,
};

use super::entity::Product;

#[async_trait]
pub trait ProductRepository: Send + Sync {
    async fn find_all(&self, query: &PaginationQuery) -> Result<(Vec<Product>, u64), AppError>;
    async fn find_by_id(&self, id: Uuid) -> Result<Product, AppError>;
    async fn create(&self, product: &Product) -> Result<Product, AppError>;
    async fn update(&self, id: Uuid, product: &Product) -> Result<Product, AppError>;
    async fn delete(&self, id: Uuid) -> Result<(), AppError>;
}

pub struct ProductServiceImpl<R: ProductRepository> {
    repository: Arc<R>,
}

impl<R: ProductRepository> ProductServiceImpl<R> {
    pub fn new(repository: Arc<R>) -> Self {
        Self { repository }
    }

    pub async fn get_all(&self, query: &PaginationQuery) -> Result<(Vec<Product>, u64), AppError> {
        self.repository.find_all(query).await
    }

    pub async fn get_by_id(&self, id: Uuid) -> Result<Product, AppError> {
        self.repository.find_by_id(id).await
    }

    pub async fn create(&self, req: CreateProductRequest) -> Result<Product, AppError> {
        let product = Product {
            id: Uuid::new_v4(),
            category_id: req.category_id.unwrap(),
            name: req.name.unwrap(),
            material: req.material.unwrap(),
            price: req.price.unwrap(),
            description: req.description.unwrap(),
            status: req.status.unwrap(),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            category: None,
        };

        self.repository.create(&product).await
    }

    pub async fn update(&self, id: Uuid, req: UpdateProductRequest) -> Result<Product, AppError> {
        let product = self.repository.find_by_id(id).await?;
        let product = Product {
            id,
            category_id: req.category_id.unwrap_or(product.category_id),
            name: req.name.unwrap_or(product.name),

            material: req.material.unwrap_or(product.material),
            price: req.price.unwrap_or(product.price),
            description: req.description.unwrap_or(product.description),
            status: req.status.unwrap_or(product.status),
            created_at: product.created_at,
            updated_at: chrono::Utc::now(),
            category: None,
        };

        self.repository.update(id, &product).await
    }

    pub async fn delete(&self, id: Uuid) -> Result<(), AppError> {
        self.repository.delete(id).await
    }
}
