use async_trait::async_trait;
use std::sync::Arc;
use uuid::Uuid;

use crate::{
    core::error::AppError,
    domain::products::dto::{CreateProductRequest, UpdateProductRequest},
    infrastructure::object_storage::s3::Storage,
    shared::dto::{pagination::PaginationQuery, response::PaginationResponse},
};

use super::entity::{Product, ProductImage};

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait ProductRepository: Send + Sync {
    async fn find_all(&self, query: &PaginationQuery) -> Result<(Vec<Product>, u64), AppError>;
    async fn find_by_id(&self, id: Uuid) -> Result<Product, AppError>;
    async fn create(&self, product: &Product) -> Result<Product, AppError>;
    async fn update(&self, id: Uuid, product: &Product) -> Result<Product, AppError>;
    async fn delete(&self, id: Uuid) -> Result<(), AppError>;
}

pub struct ProductServiceImpl {
    repository: Arc<dyn ProductRepository>,
    s3_service: Arc<dyn Storage>,
}

impl ProductServiceImpl {
    pub fn new(repository: Arc<dyn ProductRepository>, s3_service: Arc<dyn Storage>) -> Self {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::products::entity::Product;
    use crate::infrastructure::object_storage::s3::MockStorage;
    use chrono::Utc;

    #[tokio::test]
    async fn test_get_by_id() {
        let mut mock_repo = MockProductRepository::new();
        let mock_s3 = MockStorage::new();

        let id = Uuid::new_v4();
        let expected_product = Product {
            id,
            name: "Test Product".to_string(),
            price: 100.0,
            description: "Desc".to_string(),
            status: "active".to_string(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            category_ids: vec![],
            material_ids: vec![],
            categories: vec![],
            product_materials: vec![],
            images: vec![],
        };

        let product_clone = expected_product.clone();
        mock_repo
            .expect_find_by_id()
            .with(mockall::predicate::eq(id))
            .times(1)
            .returning(move |_| Ok(product_clone.clone()));

        let service = ProductServiceImpl::new(Arc::new(mock_repo), Arc::new(mock_s3));
        let result = service.get_by_id(id).await.unwrap();

        assert_eq!(result.id, expected_product.id);
        assert_eq!(result.name, expected_product.name);
    }

    #[tokio::test]
    async fn test_get_all() {
        let mut mock_repo = MockProductRepository::new();
        let mock_s3 = MockStorage::new();
        let query = PaginationQuery::default();
        let total_data = 1;
        let products = vec![Product {
            id: Uuid::new_v4(),
            name: "Test".to_string(),
            price: 100.0,
            description: "Desc".to_string(),
            status: "active".to_string(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            category_ids: vec![],
            material_ids: vec![],
            categories: vec![],
            product_materials: vec![],
            images: vec![],
        }];

        let products_clone = products.clone();
        mock_repo
            .expect_find_all()
            .times(1)
            .returning(move |_| Ok((products_clone.clone(), total_data)));

        let service = ProductServiceImpl::new(Arc::new(mock_repo), Arc::new(mock_s3));
        let result = service.get_all(&query).await.unwrap();

        assert_eq!(result.total_data, total_data);
        assert_eq!(result.data.len(), 1);
    }

    #[tokio::test]
    async fn test_create_success() {
        let mut mock_repo = MockProductRepository::new();
        let mut mock_s3 = MockStorage::new();
        let req = CreateProductRequest {
            name: "New Product".to_string(),
            price: 100.0,
            description: "Desc".to_string(),
            status: "active".to_string(),
            category_ids: vec![Uuid::new_v4()],
            material_ids: vec![Uuid::new_v4()],
            image_urls: vec!["http://example.com/image.png".to_string()],
        };

        mock_s3
            .expect_validate_object()
            .with(mockall::predicate::eq("http://example.com/image.png"))
            .times(1)
            .returning(|_| Ok(()));

        mock_repo
            .expect_create()
            .times(1)
            .returning(|product| Ok(product.clone()));

        let service = ProductServiceImpl::new(Arc::new(mock_repo), Arc::new(mock_s3));
        let result = service.create(req).await.unwrap();

        assert_eq!(result.name, "New Product");
        assert_eq!(result.images.len(), 1);
    }

    #[tokio::test]
    async fn test_create_image_not_found() {
        let mock_repo = MockProductRepository::new();
        let mut mock_s3 = MockStorage::new();
        let req = CreateProductRequest {
            name: "New Product".to_string(),
            price: 100.0,
            description: "Desc".to_string(),
            status: "active".to_string(),
            category_ids: vec![Uuid::new_v4()],
            material_ids: vec![Uuid::new_v4()],
            image_urls: vec!["http://example.com/bad.png".to_string()],
        };

        mock_s3
            .expect_validate_object()
            .with(mockall::predicate::eq("http://example.com/bad.png"))
            .times(1)
            .returning(|_| Err(AppError::NotFound("Image not found".to_string())));

        let service = ProductServiceImpl::new(Arc::new(mock_repo), Arc::new(mock_s3));
        let result = service.create(req).await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_delete() {
        let mut mock_repo = MockProductRepository::new();
        let mock_s3 = MockStorage::new();
        let id = Uuid::new_v4();

        mock_repo
            .expect_delete()
            .with(mockall::predicate::eq(id))
            .times(1)
            .returning(|_| Ok(()));

        let service = ProductServiceImpl::new(Arc::new(mock_repo), Arc::new(mock_s3));
        let result = service.delete(id).await;

        assert!(result.is_ok());
    }
}
