use async_trait::async_trait;
use std::sync::Arc;
use uuid::Uuid;

use crate::{
    core::error::AppError,
    domain::products::dto::{CreateProductRequest, UpdateProductRequest},
    infrastructure::object_storage::s3::S3Service,
    shared::dto::{pagination::PaginationQuery, response::PaginationResponse},
};

use super::entity::{Product, ProductImage};

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
    s3_service: Arc<S3Service>,
}

impl<R: ProductRepository> ProductServiceImpl<R> {
    pub fn new(repository: Arc<R>, s3_service: Arc<S3Service>) -> Self {
        Self {
            repository,
            s3_service,
        }
    }

    pub async fn get_all(
        &self,
        query: &PaginationQuery,
    ) -> Result<PaginationResponse<Vec<Product>>, AppError> {
        let (products, total_data) = self.repository.find_all(query).await?;
        let limit = query.get_limit();
        let total_page = (total_data as f64 / limit as f64).ceil() as u64;

        Ok(PaginationResponse {
            data: products,
            page: query.get_page(),
            limit,
            total_data,
            total_page,
        })
    }

    pub async fn get_by_id(&self, id: Uuid) -> Result<Product, AppError> {
        self.repository.find_by_id(id).await
    }

    pub async fn create(&self, req: CreateProductRequest) -> Result<Product, AppError> {
        // Verify all image_urls exist in S3
        for url in &req.image_urls {
            self.s3_service.validate_object(url).await?;
        }

        let id = Uuid::new_v4();
        let product = Product {
            id,
            category_ids: req.category_ids,
            material_ids: req.material_ids,
            name: req.name,
            price: req.price,
            description: req.description,
            status: req.status,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            categories: vec![],
            product_materials: vec![],
            images: req
                .image_urls
                .into_iter()
                .map(|url| ProductImage {
                    id: Uuid::new_v4(),
                    product_id: id,
                    url,
                    created_at: chrono::Utc::now(),
                    updated_at: chrono::Utc::now(),
                })
                .collect(),
        };

        self.repository.create(&product).await
    }

    pub async fn update(&self, id: Uuid, req: UpdateProductRequest) -> Result<Product, AppError> {
        // Verify all image_urls exist in S3 if provided
        if let Some(urls) = &req.image_urls {
            for url in urls {
                self.s3_service.validate_object(url).await?;
            }
        }

        let product = self.repository.find_by_id(id).await?;
        let product = Product {
            id,
            category_ids: req.category_ids.unwrap_or(product.category_ids),
            material_ids: req.material_ids.unwrap_or(product.material_ids),
            name: req.name.unwrap_or(product.name),
            price: req.price.unwrap_or(product.price),
            description: req.description.unwrap_or(product.description),
            status: req.status.unwrap_or(product.status),
            created_at: product.created_at,
            updated_at: chrono::Utc::now(),
            categories: vec![],
            product_materials: vec![],
            images: req
                .image_urls
                .map(|urls| {
                    urls.into_iter()
                        .map(|url| ProductImage {
                            id: Uuid::new_v4(),
                            product_id: id,
                            url,
                            created_at: chrono::Utc::now(),
                            updated_at: chrono::Utc::now(),
                        })
                        .collect()
                })
                .unwrap_or(product.images),
        };

        self.repository.update(id, &product).await
    }

    pub async fn delete(&self, id: Uuid) -> Result<(), AppError> {
        self.repository.delete(id).await
    }
}
