use async_trait::async_trait;
use std::sync::Arc;
use uuid::Uuid;

use crate::{
    core::error::AppError,
    domain::product_categories::dto::{CreateProductCategoryRequest, UpdateProductCategoryRequest},
    shared::dto::{pagination::PaginationQuery, response::PaginationResponse},
};

use super::entity::ProductCategory;

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
}

impl ProductCategoryServiceImpl {
    pub fn new(repository: Arc<dyn ProductCategoryRepository>) -> Self {
        Self { repository }
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
        let category = ProductCategory {
            id: Uuid::new_v4(),
            name: req.name,
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::product_categories::entity::ProductCategory;
    use chrono::Utc;

    #[tokio::test]
    async fn test_get_by_id() {
        let mut mock_repo = MockProductCategoryRepository::new();
        let id = Uuid::new_v4();
        let expected_category = ProductCategory {
            id,
            name: "Test Category".to_string(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let category_clone = expected_category.clone();
        mock_repo
            .expect_find_by_id()
            .with(mockall::predicate::eq(id))
            .times(1)
            .returning(move |_| Ok(category_clone.clone()));

        let service = ProductCategoryServiceImpl::new(Arc::new(mock_repo));
        let result = service.get_by_id(id).await.unwrap();

        assert_eq!(result.id, expected_category.id);
        assert_eq!(result.name, expected_category.name);
    }

    #[tokio::test]
    async fn test_get_all() {
        let mut mock_repo = MockProductCategoryRepository::new();
        let query = PaginationQuery::default();
        let total_data = 1;
        let categories = vec![ProductCategory {
            id: Uuid::new_v4(),
            name: "Test".to_string(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }];

        let categories_clone = categories.clone();
        mock_repo
            .expect_find_all()
            .times(1)
            .returning(move |_| Ok((categories_clone.clone(), total_data)));

        let service = ProductCategoryServiceImpl::new(Arc::new(mock_repo));
        let result = service.get_all(&query).await.unwrap();

        assert_eq!(result.total_data, total_data);
        assert_eq!(result.data.len(), 1);
    }

    #[tokio::test]
    async fn test_create() {
        let mut mock_repo = MockProductCategoryRepository::new();
        let req = CreateProductCategoryRequest {
            name: "New Category".to_string(),
        };

        mock_repo
            .expect_create()
            .times(1)
            .returning(|category| Ok(category.clone()));

        let service = ProductCategoryServiceImpl::new(Arc::new(mock_repo));
        let result = service.create(req).await.unwrap();

        assert_eq!(result.name, "New Category");
    }

    #[tokio::test]
    async fn test_update() {
        let mut mock_repo = MockProductCategoryRepository::new();
        let id = Uuid::new_v4();
        let existing = ProductCategory {
            id,
            name: "Old Name".to_string(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        let req = UpdateProductCategoryRequest {
            name: Some("New Name".to_string()),
        };

        let existing_clone = existing.clone();
        mock_repo
            .expect_find_by_id()
            .with(mockall::predicate::eq(id))
            .returning(move |_| Ok(existing_clone.clone()));

        mock_repo
            .expect_update()
            .with(mockall::predicate::eq(id), mockall::predicate::always())
            .times(1)
            .returning(|_, updated| Ok(updated.clone()));

        let service = ProductCategoryServiceImpl::new(Arc::new(mock_repo));
        let result = service.update(id, req).await.unwrap();

        assert_eq!(result.name, "New Name");
    }

    #[tokio::test]
    async fn test_delete() {
        let mut mock_repo = MockProductCategoryRepository::new();
        let id = Uuid::new_v4();

        mock_repo
            .expect_delete()
            .with(mockall::predicate::eq(id))
            .times(1)
            .returning(|_| Ok(()));

        let service = ProductCategoryServiceImpl::new(Arc::new(mock_repo));
        let result = service.delete(id).await;

        assert!(result.is_ok());
    }
}
