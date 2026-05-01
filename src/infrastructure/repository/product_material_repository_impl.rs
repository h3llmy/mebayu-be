use crate::domain::languages::entity::Language;
use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    core::error::AppError,
    domain::product_materials::{
        entity::{ProductMaterial, ProductMaterialTranslation},
        service::ProductMaterialRepository,
    },
    shared::dto::pagination::PaginationQuery,
};


pub struct ProductMaterialRepositoryImpl {
    pool: PgPool,
}

impl ProductMaterialRepositoryImpl {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl ProductMaterialRepository for ProductMaterialRepositoryImpl {
    async fn find_all(
        &self,
        query: &PaginationQuery,
        language_id: Option<Uuid>,
    ) -> Result<(Vec<ProductMaterial>, u64), AppError> {
        let limit = query.get_limit() as i64;
        let offset = query.get_offset();
        let search = query.get_search().map(|s| format!("%{}%", s));

        let languages = sqlx::query_as!(
            Language,
            "SELECT id, code, name, is_default, created_at, updated_at FROM languages"
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

        let rows = sqlx::query!(
            r#"
            SELECT pm.id, pm.created_at, pm.updated_at, COUNT(*) OVER() as total_count
            FROM product_materials pm
            WHERE ($1::text IS NULL OR EXISTS (
                SELECT 1 FROM product_material_translations pmt 
                WHERE pmt.material_id = pm.id AND pmt.name ILIKE $1
            ))
            ORDER BY pm.created_at DESC
            LIMIT $2 OFFSET $3
            "#,
            search,
            limit,
            offset
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

        let total = rows.first().map(|r| r.total_count.unwrap_or(0)).unwrap_or(0);
        
        let mut materials = Vec::new();
        for row in rows {
            let translation_rows = sqlx::query!(
                "SELECT material_id, language_id, name FROM product_material_translations 
                 WHERE material_id = $1 AND ($2::uuid IS NULL OR language_id = $2)",
                row.id,
                language_id
            )
            .fetch_all(&self.pool)
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

            let translations: Vec<ProductMaterialTranslation> = translation_rows.into_iter().map(|t_row| {
                ProductMaterialTranslation {
                    material_id: t_row.material_id,
                    language_id: t_row.language_id,
                    language: languages.iter().find(|l| l.id == t_row.language_id).cloned(),
                    name: t_row.name,
                }
            }).collect();

            materials.push(ProductMaterial {
                id: row.id,
                created_at: row.created_at,
                updated_at: row.updated_at,
                translations,
            });
        }

        Ok((materials, total as u64))
    }

    async fn find_by_id(&self, id: Uuid, language_id: Option<Uuid>) -> Result<ProductMaterial, AppError> {
        let row = sqlx::query!(
            "SELECT id, created_at, updated_at FROM product_materials WHERE id = $1",
            id
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?
        .ok_or_else(|| AppError::NotFound("Product material not found".to_string()))?;

        let languages = sqlx::query_as!(
            Language,
            "SELECT id, code, name, is_default, created_at, updated_at FROM languages"
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

        let translation_rows = sqlx::query!(
            "SELECT material_id, language_id, name FROM product_material_translations 
             WHERE material_id = $1 AND ($2::uuid IS NULL OR language_id = $2)",
            id,
            language_id
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

        let translations: Vec<ProductMaterialTranslation> = translation_rows.into_iter().map(|t_row| {
            ProductMaterialTranslation {
                material_id: t_row.material_id,
                language_id: t_row.language_id,
                language: languages.iter().find(|l| l.id == t_row.language_id).cloned(),
                name: t_row.name,
            }
        }).collect();

        Ok(ProductMaterial {
            id: row.id,
            created_at: row.created_at,
            updated_at: row.updated_at,
            translations,
        })
    }

    async fn create(&self, material: &ProductMaterial) -> Result<ProductMaterial, AppError> {
        let mut tx = self.pool.begin().await.map_err(|e| AppError::Database(e.to_string()))?;

        sqlx::query!(
            "INSERT INTO product_materials (id, created_at, updated_at) VALUES ($1, $2, $3)",
            material.id,
            material.created_at,
            material.updated_at
        )
        .execute(&mut *tx)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

        for translation in &material.translations {
            sqlx::query!(
                "INSERT INTO product_material_translations (material_id, language_id, name) VALUES ($1, $2, $3)",
                material.id,
                translation.language_id,
                translation.name
            )
            .execute(&mut *tx)
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;
        }

        tx.commit().await.map_err(|e| AppError::Database(e.to_string()))?;

        self.find_by_id(material.id, None).await
    }

    async fn update(
        &self,
        id: Uuid,
        material: &ProductMaterial,
    ) -> Result<ProductMaterial, AppError> {
        let mut tx = self.pool.begin().await.map_err(|e| AppError::Database(e.to_string()))?;

        sqlx::query!(
            "UPDATE product_materials SET updated_at = $2 WHERE id = $1",
            id,
            material.updated_at
        )
        .execute(&mut *tx)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

        sqlx::query!("DELETE FROM product_material_translations WHERE material_id = $1", id)
            .execute(&mut *tx)
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

        for translation in &material.translations {
            sqlx::query!(
                "INSERT INTO product_material_translations (material_id, language_id, name) VALUES ($1, $2, $3)",
                id,
                translation.language_id,
                translation.name
            )
            .execute(&mut *tx)
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;
        }

        tx.commit().await.map_err(|e| AppError::Database(e.to_string()))?;

        self.find_by_id(id, None).await
    }

    async fn delete(&self, id: Uuid) -> Result<(), AppError> {
        sqlx::query("DELETE FROM product_materials WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::infrastructure::database::migrations::run_migrations;

    use super::*;
    use chrono::Utc;
    use sqlx::PgPool;
    use uuid::Uuid;

    const LANGUAGE_ID: Uuid = Uuid::from_u128(1);

    async fn setup_db(pool: &PgPool) {
        run_migrations(pool).await;
        // Insert a test language
        sqlx::query(
            "INSERT INTO languages (id, code, name, is_default) VALUES ($1, $2, $3, $4) ON CONFLICT DO NOTHING"
        )
        .bind(LANGUAGE_ID)
        .bind("maten")
        .bind("English")
        .bind(true)
        .execute(pool)
        .await
        .unwrap();
    }

    fn sample_material(name: &str) -> ProductMaterial {
        let id = Uuid::new_v4();
        ProductMaterial {
            id,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            translations: vec![ProductMaterialTranslation {
                material_id: id,
                language_id: LANGUAGE_ID,
                language: None,
                name: name.to_string(),
            }],
        }
    }

    #[sqlx::test]
    async fn test_create_and_find_by_id(pool: PgPool) {
        setup_db(&pool).await;
        let repo = ProductMaterialRepositoryImpl::new(pool.clone());

        let material = sample_material("Steel");

        let created = repo.create(&material).await.unwrap();
        assert_eq!(created.translations[0].name, "Steel");

        let found = repo.find_by_id(material.id, None).await.unwrap();
        assert_eq!(found.id, material.id);
        assert_eq!(found.translations[0].name, "Steel");
    }

    #[sqlx::test]
    async fn test_find_by_id_not_found(pool: PgPool) {
        setup_db(&pool).await;
        let repo = ProductMaterialRepositoryImpl::new(pool.clone());

        let result = repo.find_by_id(Uuid::new_v4(), None).await;
        assert!(result.is_err());
    }

    #[sqlx::test]
    async fn test_find_all(pool: PgPool) {
        setup_db(&pool).await;
        let repo = ProductMaterialRepositoryImpl::new(pool.clone());

        for i in 0..3 {
            repo.create(&sample_material(&format!("Material {}", i)))
                .await
                .unwrap();
        }

        let query = PaginationQuery {
            page: Some(1),
            search: None,
            limit: Some(10),
            sort: None,
            sort_order: None,
        };

        let (items, total) = repo.find_all(&query, None).await.unwrap();

        assert_eq!(total, 3);
        assert_eq!(items.len(), 3);
    }

    #[sqlx::test]
    async fn test_update(pool: PgPool) {
        setup_db(&pool).await;
        let repo = ProductMaterialRepositoryImpl::new(pool.clone());

        let mut material = sample_material("Old Name");
        repo.create(&material).await.unwrap();

        material.translations[0].name = "New Name".to_string();
        material.updated_at = Utc::now();

        let updated = repo.update(material.id, &material).await.unwrap();
        assert_eq!(updated.translations[0].name, "New Name");
    }

    #[sqlx::test]
    async fn test_delete(pool: PgPool) {
        setup_db(&pool).await;
        let repo = ProductMaterialRepositoryImpl::new(pool.clone());

        let material = sample_material("DeleteMe");
        repo.create(&material).await.unwrap();

        repo.delete(material.id).await.unwrap();

        let result = repo.find_by_id(material.id, None).await;
        assert!(result.is_err());
    }
}
