use async_trait::async_trait;
use std::sync::Arc;
use uuid::Uuid;

use crate::{
    core::error::AppError,
    domain::products::dto::{CreateProductRequest, UpdateProductRequest, GetProductsQuery},
    infrastructure::object_storage::s3::Storage,
    shared::dto::response::PaginationResponse,
};

use super::entity::{Product, ProductImage, ProductTranslation};

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait ProductRepository: Send + Sync {
    async fn find_all(
        &self,
        query: &GetProductsQuery,
        language_id: Option<Uuid>,
    ) -> Result<(Vec<Product>, u64), AppError>;
    async fn find_by_id(&self, id: Uuid, language_id: Option<Uuid>) -> Result<Product, AppError>;
    async fn find_recommendations(
        &self,
        id: Uuid,
        limit: i64,
        language_id: Option<Uuid>,
    ) -> Result<Vec<Product>, AppError>;
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
        query: &GetProductsQuery,
        language_id: Option<Uuid>,
    ) -> Result<PaginationResponse<Vec<Product>>, AppError> {
        let (products, total_data) = self.repository.find_all(query, language_id).await?;
        let limit = query.pagination.get_limit();
        let total_page = (total_data as f64 / limit as f64).ceil() as u64;

        Ok(PaginationResponse {
            data: products,
            page: query.pagination.get_page(),
            limit,
            total_data,
            total_page,
        })
    }

    pub async fn get_by_id(&self, id: Uuid, language_id: Option<Uuid>) -> Result<Product, AppError> {
        self.repository.find_by_id(id, language_id).await
    }

    pub async fn create(&self, req: CreateProductRequest) -> Result<Product, AppError> {
        for url in &req.image_urls {
            self.s3_service.validate_object(url).await?;
        }

        let id = Uuid::new_v4();
        let product = Product {
            id,
            category_ids: req.category_ids,
            material_ids: req.material_ids,
            foundation_ids: req.foundation_ids,
            price: req.price,
            status: req.status,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            product_categories: vec![],
            product_materials: vec![],
            product_foundations: vec![],
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
            translations: req.translations.into_iter().map(|t| ProductTranslation {
                product_id: id, language: None,
                language_id: t.language_id,
                name: t.name,
                description: t.description,
            }).collect(),
        };

        self.repository.create(&product).await?;
        self.repository.find_by_id(id, None).await
    }

    pub async fn update(&self, id: Uuid, req: UpdateProductRequest) -> Result<Product, AppError> {
        if let Some(urls) = &req.image_urls {
            for url in urls {
                self.s3_service.validate_object(url).await?;
            }
        }

        let product = self.repository.find_by_id(id, None).await?;
        let updated_product = Product {
            id,
            category_ids: req.category_ids.unwrap_or(product.category_ids),
            material_ids: req.material_ids.unwrap_or(product.material_ids),
            foundation_ids: req.foundation_ids.unwrap_or(product.foundation_ids),
            price: req.price.unwrap_or(product.price),
            status: req.status.unwrap_or(product.status),
            created_at: product.created_at,
            updated_at: chrono::Utc::now(),
            product_categories: vec![],
            product_materials: vec![],
            product_foundations: vec![],
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
            translations: req.translations.map(|trs| {
                trs.into_iter().map(|t| ProductTranslation {
                    product_id: id, language: None,
                    language_id: t.language_id,
                    name: t.name,
                    description: t.description,
                }).collect()
            }).unwrap_or(product.translations),
        };

        self.repository.update(id, &updated_product).await?;
        self.repository.find_by_id(id, None).await
    }

    pub async fn get_recommendations(
        &self,
        id: Uuid,
        limit: Option<i64>,
        language_id: Option<Uuid>,
    ) -> Result<Vec<Product>, AppError> {
        self.repository.find_by_id(id, language_id).await?;
        let limit = limit.unwrap_or(8).min(50).max(1);
        self.repository.find_recommendations(id, limit, language_id).await
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

    fn test_lang_id() -> Uuid { Uuid::new_v4() }

    #[tokio::test]
    async fn test_get_by_id() {
        let mut mock_repo = MockProductRepository::new();
        let mock_s3 = MockStorage::new();

        let id = Uuid::new_v4();
        let expected_product = Product {
            id,
            price: 100.0,
            status: "active".to_string(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            category_ids: vec![],
            material_ids: vec![],
            foundation_ids: vec![],
            product_categories: vec![],
            product_materials: vec![],
            product_foundations: vec![],
            images: vec![],
            translations: vec![ProductTranslation {
                product_id: id, language: None,
                language_id: test_lang_id(),
                name: "Test Product".to_string(),
                description: "Desc".to_string(),
            }],
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
        assert_eq!(result.translations[0].name, "Test Product");
    }

    #[tokio::test]
    async fn test_get_all() {
        let mut mock_repo = MockProductRepository::new();
        let mock_s3 = MockStorage::new();
        let query = GetProductsQuery::default();
        let total_data = 1;
        let id = Uuid::new_v4();
        let products = vec![Product {
            id,
            price: 100.0,
            status: "active".to_string(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            category_ids: vec![],
            material_ids: vec![],
            foundation_ids: vec![],
            product_categories: vec![],
            product_materials: vec![],
            product_foundations: vec![],
            images: vec![],
            translations: vec![ProductTranslation {
                product_id: id, language: None,
                language_id: test_lang_id(),
                name: "Test".to_string(),
                description: "Desc".to_string(),
            }],
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
}
