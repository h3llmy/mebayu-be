use async_trait::async_trait;
use std::sync::Arc;
use uuid::Uuid;

use crate::{
    core::error::AppError,
    domain::product_materials::dto::{CreateProductMaterialRequest, UpdateProductMaterialRequest},
    shared::dto::{pagination::PaginationQuery, response::PaginationResponse},
};

use super::entity::ProductMaterial;

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
}

impl ProductMaterialServiceImpl {
    pub fn new(repository: Arc<dyn ProductMaterialRepository>) -> Self {
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

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    #[tokio::test]
    async fn test_get_by_id() {
        let mut mock_repo = MockProductMaterialRepository::new();
        let id = Uuid::new_v4();
        let expected_material = ProductMaterial {
            id,
            name: "Test Material".to_string(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let material_clone = expected_material.clone();
        mock_repo
            .expect_find_by_id()
            .with(mockall::predicate::eq(id))
            .times(1)
            .returning(move |_| Ok(material_clone.clone()));

        let service = ProductMaterialServiceImpl::new(Arc::new(mock_repo));
        let result = service.get_by_id(id).await.unwrap();

        assert_eq!(result.id, expected_material.id);
        assert_eq!(result.name, expected_material.name);
    }

    #[tokio::test]
    async fn test_get_all() {
        let mut mock_repo = MockProductMaterialRepository::new();
        let query = PaginationQuery::default();
        let total_data = 1;
        let materials = vec![ProductMaterial {
            id: Uuid::new_v4(),
            name: "Test".to_string(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }];

        let materials_clone = materials.clone();
        mock_repo
            .expect_find_all()
            .times(1)
            .returning(move |_| Ok((materials_clone.clone(), total_data)));

        let service = ProductMaterialServiceImpl::new(Arc::new(mock_repo));
        let result = service.get_all(&query).await.unwrap();

        assert_eq!(result.total_data, total_data);
        assert_eq!(result.data.len(), 1);
    }

    #[tokio::test]
    async fn test_create() {
        let mut mock_repo = MockProductMaterialRepository::new();
        let req = CreateProductMaterialRequest {
            name: "New Material".to_string(),
        };

        mock_repo
            .expect_create()
            .times(1)
            .returning(|material| Ok(material.clone()));

        let service = ProductMaterialServiceImpl::new(Arc::new(mock_repo));
        let result = service.create(req).await.unwrap();

        assert_eq!(result.name, "New Material");
    }

    #[tokio::test]
    async fn test_update() {
        let mut mock_repo = MockProductMaterialRepository::new();
        let id = Uuid::new_v4();
        let existing = ProductMaterial {
            id,
            name: "Old Name".to_string(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        let req = UpdateProductMaterialRequest {
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

        let service = ProductMaterialServiceImpl::new(Arc::new(mock_repo));
        let result = service.update(id, req).await.unwrap();

        assert_eq!(result.name, "New Name");
    }

    #[tokio::test]
    async fn test_delete() {
        let mut mock_repo = MockProductMaterialRepository::new();
        let id = Uuid::new_v4();

        mock_repo
            .expect_delete()
            .with(mockall::predicate::eq(id))
            .times(1)
            .returning(|_| Ok(()));

        let service = ProductMaterialServiceImpl::new(Arc::new(mock_repo));
        let result = service.delete(id).await;

        assert!(result.is_ok());
    }
}
