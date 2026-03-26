use std::collections::HashMap;

use async_trait::async_trait;
use sqlx::{PgPool, Row};
use uuid::Uuid;

use crate::{
    core::error::AppError,
    domain::{
        product_categories::entity::ProductCategory,
        product_foundations::entity::ProductFoundation,
        product_materials::entity::ProductMaterial,
        products::{
            dto::GetProductsQuery,
            entity::{Product, ProductImage},
            service::ProductRepository,
        },
    },
    shared::dto::pagination::SortOrder,
};

pub struct ProductRepositoryImpl {
    pool: PgPool,
}

impl ProductRepositoryImpl {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl ProductRepository for ProductRepositoryImpl {
    async fn find_all(&self, query: &GetProductsQuery) -> Result<(Vec<Product>, u64), AppError> {
        let limit = query.pagination.get_limit() as i64;
        let offset = query.pagination.get_offset();

        let search = query.pagination.get_search().map(|s| format!("%{}%", s));

        let allowed_sort_fields = ["name", "price", "created_at", "updated_at", "status"];

        let sort_field = query
            .pagination
            .get_sort()
            .filter(|field| allowed_sort_fields.contains(&field.as_str()))
            .unwrap_or_else(|| "created_at".to_string());

        let sort_order = match query.pagination.get_sort_order() {
            Some(SortOrder::Asc) => "ASC",
            _ => "DESC",
        };

        let mut where_clauses = Vec::new();
        let mut param_index = 3;

        if search.is_some() {
            where_clauses.push(format!(
                "(p.name ILIKE ${} OR pm.name ILIKE ${} OR pf.name ILIKE ${} OR p.description ILIKE ${})",
                param_index, param_index, param_index, param_index
            ));
            param_index += 1;
        }

        if query.category_id.is_some() {
            where_clauses.push(format!(
                "EXISTS (SELECT 1 FROM product_category_relations WHERE product_id = p.id AND category_id = ${})",
                param_index
            ));
            param_index += 1;
        }

        if query.material_id.is_some() {
            where_clauses.push(format!(
                "EXISTS (SELECT 1 FROM product_material_relations WHERE product_id = p.id AND material_id = ${})",
                param_index
            ));
            param_index += 1;
        }

        if query.foundation_id.is_some() {
            where_clauses.push(format!(
                "EXISTS (SELECT 1 FROM product_foundation_relations WHERE product_id = p.id AND foundation_id = ${})",
                param_index
            ));
            // param_index += 1;
        }

        let where_clause = if where_clauses.is_empty() {
            "".to_string()
        } else {
            format!("WHERE {}", where_clauses.join(" AND "))
        };

        let sql = format!(
            r#"
            SELECT
                p.*,
                COUNT(*) OVER() as total_count,

                COALESCE(
                    JSON_AGG(DISTINCT pc.*)
                    FILTER (WHERE pc.id IS NOT NULL),
                    '[]'
                ) as categories,

                COALESCE(
                    JSON_AGG(DISTINCT pm.*)
                    FILTER (WHERE pm.id IS NOT NULL),
                    '[]'
                ) as materials,

                COALESCE(
                    JSON_AGG(DISTINCT pf.*)
                    FILTER (WHERE pf.id IS NOT NULL),
                    '[]'
                ) as foundations,

                COALESCE(
                    JSON_AGG(DISTINCT pi.*)
                    FILTER (WHERE pi.id IS NOT NULL),
                    '[]'
                ) as images

            FROM products p

            LEFT JOIN product_category_relations pcr
                ON p.id = pcr.product_id
            LEFT JOIN product_categories pc
                ON pcr.category_id = pc.id

            LEFT JOIN product_material_relations pmr
                ON p.id = pmr.product_id
            LEFT JOIN product_materials pm
                ON pmr.material_id = pm.id

            LEFT JOIN product_foundation_relations pfr
                ON p.id = pfr.product_id
            LEFT JOIN product_foundations pf
                ON pfr.foundation_id = pf.id

            LEFT JOIN product_images pi
                ON p.id = pi.product_id

            {}
            GROUP BY p.id
            ORDER BY p.{} {}
            LIMIT $1 OFFSET $2
            "#,
            where_clause, sort_field, sort_order
        );

        let mut sql_query = sqlx::query(&sql).bind(limit).bind(offset);

        if let Some(s) = search {
            sql_query = sql_query.bind(s);
        }

        if let Some(cid) = query.category_id {
            sql_query = sql_query.bind(cid);
        }

        if let Some(mid) = query.material_id {
            sql_query = sql_query.bind(mid);
        }

        if let Some(fid) = query.foundation_id {
            sql_query = sql_query.bind(fid);
        }

        use crate::core::monitoring::observe_db;
        let rows = observe_db("product.find_all", sql_query.fetch_all(&self.pool))
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

        if rows.is_empty() {
            return Ok((vec![], 0));
        }

        let total = rows[0].get::<i64, _>("total_count") as u64;

        let products = rows
            .into_iter()
            .map(|r| {
                let categories: Vec<ProductCategory> =
                    serde_json::from_value(r.get("categories")).unwrap_or_default();

                let materials: Vec<ProductMaterial> =
                    serde_json::from_value(r.get("materials")).unwrap_or_default();

                let foundations: Vec<ProductFoundation> =
                    serde_json::from_value(r.get("foundations")).unwrap_or_default();

                let images: Vec<ProductImage> =
                    serde_json::from_value(r.get("images")).unwrap_or_default();

                Product {
                    id: r.get("id"),
                    name: r.get("name"),
                    price: r.get("price"),
                    description: r.get("description"),
                    status: r.get("status"),
                    created_at: r.get("created_at"),
                    updated_at: r.get("updated_at"),
                    category_ids: categories.iter().map(|c| c.id).collect(),
                    material_ids: materials.iter().map(|m| m.id).collect(),
                    foundation_ids: foundations.iter().map(|f| f.id).collect(),
                    categories,
                    product_foundations: foundations,
                    product_materials: materials,
                    images,
                }
            })
            .collect();

        Ok((products, total))
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Product, AppError> {
        use crate::core::monitoring::observe_db;
        let row = observe_db("product.find_by_id", sqlx::query!(
            r#"
        SELECT 
            p.*,

            COALESCE(
                json_agg(DISTINCT pc) 
                FILTER (WHERE pc.id IS NOT NULL),
                '[]'
            ) as "categories!: serde_json::Value",

            COALESCE(
                json_agg(DISTINCT pm) 
                FILTER (WHERE pm.id IS NOT NULL),
                '[]'
            ) as "product_materials!: serde_json::Value",

            COALESCE(
                json_agg(DISTINCT pf) 
                FILTER (WHERE pf.id IS NOT NULL),
                '[]'
            ) as "product_foundations!: serde_json::Value",

            COALESCE(
                json_agg(DISTINCT pi) 
                FILTER (WHERE pi.id IS NOT NULL),
                '[]'
            ) as "images!: serde_json::Value"

        FROM products p

        LEFT JOIN product_category_relations pcr 
            ON p.id = pcr.product_id
        LEFT JOIN product_categories pc 
            ON pcr.category_id = pc.id

        LEFT JOIN product_material_relations pmr 
            ON p.id = pmr.product_id
        LEFT JOIN product_materials pm 
            ON pmr.material_id = pm.id

        LEFT JOIN product_foundation_relations pfr 
            ON p.id = pfr.product_id
        LEFT JOIN product_foundations pf 
            ON pfr.foundation_id = pf.id

        LEFT JOIN product_images pi 
            ON p.id = pi.product_id

        WHERE p.id = $1
        GROUP BY p.id
        "#,
            id
        )
        .fetch_optional(&self.pool))
        .await
        .map_err(|e| AppError::Database(e.to_string()))?
        .ok_or_else(|| AppError::NotFound("Product not found".to_string()))?;

        let categories: Vec<ProductCategory> =
            serde_json::from_value(row.categories).unwrap_or_default();

        let product_materials: Vec<ProductMaterial> =
            serde_json::from_value(row.product_materials).unwrap_or_default();

        let product_foundations: Vec<ProductFoundation> =
            serde_json::from_value(row.product_foundations).unwrap_or_default();

        let images: Vec<ProductImage> = serde_json::from_value(row.images).unwrap_or_default();

        Ok(Product {
            id: row.id,
            name: row.name,
            price: row.price,
            description: row.description,
            status: row.status,
            created_at: row.created_at,
            updated_at: row.updated_at,
            category_ids: categories.iter().map(|c| c.id).collect(),
            material_ids: product_materials.iter().map(|m| m.id).collect(),
            foundation_ids: product_foundations.iter().map(|f| f.id).collect(),
            categories,
            product_foundations,
            product_materials,
            images,
        })
    }

    async fn find_recommendations(&self, id: Uuid, limit: i64) -> Result<Vec<Product>, AppError> {
        let sql = r#"
            SELECT
                p.*,

                COALESCE(
                    JSON_AGG(DISTINCT pc.*)
                    FILTER (WHERE pc.id IS NOT NULL),
                    '[]'
                ) as categories,

                COALESCE(
                    JSON_AGG(DISTINCT pm.*)
                    FILTER (WHERE pm.id IS NOT NULL),
                    '[]'
                ) as materials,

                COALESCE(
                    JSON_AGG(DISTINCT pf.*)
                    FILTER (WHERE pf.id IS NOT NULL),
                    '[]'
                ) as foundations,

                COALESCE(
                    JSON_AGG(DISTINCT pi.*)
                    FILTER (WHERE pi.id IS NOT NULL),
                    '[]'
                ) as images,

                COUNT(DISTINCT pcr2.category_id) + COUNT(DISTINCT pmr2.material_id) + COUNT(DISTINCT pfr2.foundation_id) AS overlap_score

            FROM products p

            LEFT JOIN product_category_relations pcr ON p.id = pcr.product_id
            LEFT JOIN product_categories pc ON pcr.category_id = pc.id

            LEFT JOIN product_material_relations pmr ON p.id = pmr.product_id
            LEFT JOIN product_materials pm ON pmr.material_id = pm.id

            LEFT JOIN product_foundation_relations pfr ON p.id = pfr.product_id
            LEFT JOIN product_foundations pf ON pfr.foundation_id = pf.id

            LEFT JOIN product_images pi ON p.id = pi.product_id

            -- join to find shared categories
            LEFT JOIN product_category_relations pcr2
                ON pcr2.product_id = p.id
                AND pcr2.category_id IN (
                    SELECT category_id FROM product_category_relations WHERE product_id = $1
                )

            -- join to find shared materials
            LEFT JOIN product_material_relations pmr2
                ON pmr2.product_id = p.id
                AND pmr2.material_id IN (
                    SELECT material_id FROM product_material_relations WHERE product_id = $1
                )

            -- join to find shared foundations
            LEFT JOIN product_foundation_relations pfr2
                ON pfr2.product_id = p.id
                AND pfr2.foundation_id IN (
                    SELECT foundation_id FROM product_foundation_relations WHERE product_id = $1
                )

            WHERE p.id != $1
              AND (
                    pcr2.category_id IS NOT NULL
                 OR pmr2.material_id IS NOT NULL
                 OR pfr2.foundation_id IS NOT NULL
              )

            GROUP BY p.id
            ORDER BY overlap_score DESC, p.created_at DESC
            LIMIT $2
        "#;

        let rows = sqlx::query(sql)
            .bind(id)
            .bind(limit)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

        let products = rows
            .into_iter()
            .map(|r| {
                let categories: Vec<crate::domain::product_categories::entity::ProductCategory> =
                    serde_json::from_value(r.get("categories")).unwrap_or_default();

                let materials: Vec<crate::domain::product_materials::entity::ProductMaterial> =
                    serde_json::from_value(r.get("materials")).unwrap_or_default();

                let foundations: Vec<crate::domain::product_foundations::entity::ProductFoundation> =
                    serde_json::from_value(r.get("foundations")).unwrap_or_default();

                let images: Vec<ProductImage> =
                    serde_json::from_value(r.get("images")).unwrap_or_default();

                Product {
                    id: r.get("id"),
                    name: r.get("name"),
                    price: r.get("price"),
                    description: r.get("description"),
                    status: r.get("status"),
                    created_at: r.get("created_at"),
                    updated_at: r.get("updated_at"),
                    category_ids: categories.iter().map(|c| c.id).collect(),
                    material_ids: materials.iter().map(|m| m.id).collect(),
                    foundation_ids: foundations.iter().map(|f| f.id).collect(),
                    categories,
                    product_foundations: foundations,
                    product_materials: materials,
                    images,
                }
            })
            .collect();

        Ok(products)
    }

    async fn create(&self, product: &Product) -> Result<Product, AppError> {
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e: sqlx::Error| AppError::Database(e.to_string()))?;

        // 1. Check if all categories exist
        if !product.category_ids.is_empty() {
            let count = sqlx::query!(
                "SELECT count(*) FROM product_categories WHERE id = ANY($1)",
                &product.category_ids
            )
            .fetch_one(&mut *tx)
            .await
            .map_err(|e: sqlx::Error| AppError::Database(e.to_string()))?
            .count
            .unwrap_or(0);

            if count != product.category_ids.len() as i64 {
                return Err(AppError::NotFound(
                    "One or more categories not found".to_string(),
                ));
            }
        } else {
            return Err(AppError::Validation(HashMap::from([(
                "category_ids".to_string(),
                vec!["At least one category is required".to_string()],
            )])));
        }

        // 2. Insert product
        sqlx::query!(
            "INSERT INTO products (id, name, price, description, status, created_at, updated_at)
             VALUES ($1,$2,$3,$4,$5,$6,$7)",
            product.id,
            product.name,
            product.price,
            product.description,
            product.status,
            product.created_at,
            product.updated_at
        )
        .execute(&mut *tx)
        .await
        .map_err(|e: sqlx::Error| AppError::Database(e.to_string()))?;

        // 3. Insert category relations
        for category_id in &product.category_ids {
            sqlx::query!(
                "INSERT INTO product_category_relations (product_id, category_id) VALUES ($1, $2)",
                product.id,
                category_id
            )
            .execute(&mut *tx)
            .await
            .map_err(|e: sqlx::Error| AppError::Database(e.to_string()))?;
        }

        // 4. Insert material relations
        for material_id in &product.material_ids {
            sqlx::query!(
                "INSERT INTO product_material_relations (product_id, material_id) VALUES ($1, $2)",
                product.id,
                material_id
            )
            .execute(&mut *tx)
            .await
            .map_err(|e: sqlx::Error| AppError::Database(e.to_string()))?;
        }

        // 5. Insert foundation relations
        for foundation_id in &product.foundation_ids {
            sqlx::query!(
                "INSERT INTO product_foundation_relations (product_id, foundation_id) VALUES ($1, $2)",
                product.id,
                foundation_id
            )
            .execute(&mut *tx)
            .await
            .map_err(|e: sqlx::Error| AppError::Database(e.to_string()))?;
        }

        // 6. Insert image relations
        for image in &product.images {
            sqlx::query(
                "INSERT INTO product_images (id, product_id, url, created_at, updated_at) VALUES ($1, $2, $3, $4, $5)",
            )
            .bind(image.id)
            .bind(product.id)
            .bind(&image.url)
            .bind(image.created_at)
            .bind(image.updated_at)
            .execute(&mut *tx)
            .await
            .map_err(|e: sqlx::Error| AppError::Database(e.to_string()))?;
        }

        tx.commit()
            .await
            .map_err(|e: sqlx::Error| AppError::Database(e.to_string()))?;

        self.find_by_id(product.id).await
    }

    async fn update(&self, id: Uuid, product: &Product) -> Result<Product, AppError> {
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e: sqlx::Error| AppError::Database(e.to_string()))?;

        // 1. Update product basic fields
        sqlx::query!(
            "UPDATE products SET name = $2, price = $3, description = $4, status = $5, updated_at = $6 WHERE id = $1",
            id,
            product.name,
            product.price,
            product.description,
            product.status,
            product.updated_at
        )
        .execute(&mut *tx)
        .await
        .map_err(|e: sqlx::Error| AppError::Database(e.to_string()))?;

        // 2. Update category relations
        // Clear existing
        sqlx::query!(
            "DELETE FROM product_category_relations WHERE product_id = $1",
            id
        )
        .execute(&mut *tx)
        .await
        .map_err(|e: sqlx::Error| AppError::Database(e.to_string()))?;

        // Add new
        for category_id in &product.category_ids {
            sqlx::query!(
                "INSERT INTO product_category_relations (product_id, category_id) VALUES ($1, $2)",
                id,
                category_id
            )
            .execute(&mut *tx)
            .await
            .map_err(|e: sqlx::Error| AppError::Database(e.to_string()))?;
        }

        // 3. Update material relations
        // Clear existing
        sqlx::query!(
            "DELETE FROM product_material_relations WHERE product_id = $1",
            id
        )
        .execute(&mut *tx)
        .await
        .map_err(|e: sqlx::Error| AppError::Database(e.to_string()))?;

        // Add new
        for material_id in &product.material_ids {
            sqlx::query!(
                "INSERT INTO product_material_relations (product_id, material_id) VALUES ($1, $2)",
                id,
                material_id
            )
            .execute(&mut *tx)
            .await
            .map_err(|e: sqlx::Error| AppError::Database(e.to_string()))?;
        }

        // 4. Update foundation relations
        // Clear existing
        sqlx::query!(
            "DELETE FROM product_foundation_relations WHERE product_id = $1",
            id
        )
        .execute(&mut *tx)
        .await
        .map_err(|e: sqlx::Error| AppError::Database(e.to_string()))?;

        // Add new
        for foundation_id in &product.foundation_ids {
            sqlx::query!(
                "INSERT INTO product_foundation_relations (product_id, foundation_id) VALUES ($1, $2)",
                id,
                foundation_id
            )
            .execute(&mut *tx)
            .await
            .map_err(|e: sqlx::Error| AppError::Database(e.to_string()))?;
        }

        // 5. Update image relations
        // Clear existing
        sqlx::query("DELETE FROM product_images WHERE product_id = $1")
            .bind(id)
            .execute(&mut *tx)
            .await
            .map_err(|e: sqlx::Error| AppError::Database(e.to_string()))?;

        // Add new
        for image in &product.images {
            sqlx::query(
                "INSERT INTO product_images (id, product_id, url, created_at, updated_at) VALUES ($1, $2, $3, $4, $5)",
            )
            .bind(image.id)
            .bind(id)
            .bind(&image.url)
            .bind(image.created_at)
            .bind(image.updated_at)
            .execute(&mut *tx)
            .await
            .map_err(|e: sqlx::Error| AppError::Database(e.to_string()))?;
        }

        tx.commit()
            .await
            .map_err(|e: sqlx::Error| AppError::Database(e.to_string()))?;

        self.find_by_id(id).await
    }

    async fn delete(&self, id: Uuid) -> Result<(), AppError> {
        sqlx::query!("DELETE FROM products WHERE id = $1", id)
            .execute(&self.pool)
            .await
            .map_err(|e: sqlx::Error| AppError::Database(e.to_string()))?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        infrastructure::database::migrations::run_migrations,
        shared::dto::pagination::PaginationQuery,
    };
    use chrono::Utc;

    async fn setup_db(pool: &PgPool) {
        run_migrations(pool).await;
    }

    async fn seed_category(pool: &PgPool) -> ProductCategory {
        let category = ProductCategory {
            id: Uuid::new_v4(),
            name: "Category 1".to_string(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        sqlx::query!(
            "INSERT INTO product_categories (id,name,created_at,updated_at)
             VALUES ($1,$2,$3,$4)",
            category.id,
            category.name,
            category.created_at,
            category.updated_at
        )
        .execute(pool)
        .await
        .unwrap();

        category
    }

    async fn seed_material(pool: &PgPool) -> ProductMaterial {
        let material = ProductMaterial {
            id: Uuid::new_v4(),
            name: "Material 1".to_string(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        sqlx::query!(
            "INSERT INTO product_materials (id,name,created_at,updated_at)
             VALUES ($1,$2,$3,$4)",
            material.id,
            material.name,
            material.created_at,
            material.updated_at
        )
        .execute(pool)
        .await
        .unwrap();

        material
    }

    async fn seed_foundation(pool: &PgPool) -> ProductFoundation {
        let foundation = ProductFoundation {
            id: Uuid::new_v4(),
            name: "Foundation 1".to_string(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        sqlx::query!(
            "INSERT INTO product_foundations (id,name,created_at,updated_at)
             VALUES ($1,$2,$3,$4)",
            foundation.id,
            foundation.name,
            foundation.created_at,
            foundation.updated_at
        )
        .execute(pool)
        .await
        .unwrap();

        foundation
    }

    fn sample_product(category_id: Uuid, material_id: Uuid, foundation_id: Uuid) -> Product {
        Product {
            id: Uuid::new_v4(),
            name: "Product 1".to_string(),
            price: 100.0,
            description: "Test product".to_string(),
            status: "ACTIVE".to_string(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            category_ids: vec![category_id],
            material_ids: vec![material_id],
            foundation_ids: vec![foundation_id],
            categories: vec![],
            product_materials: vec![],
            product_foundations: vec![],
            images: vec![],
        }
    }

    #[sqlx::test]
    async fn test_create_and_find_by_id(pool: PgPool) {
        setup_db(&pool).await;
        let repo = ProductRepositoryImpl::new(pool.clone());

        let category = seed_category(&pool).await;
        let material = seed_material(&pool).await;
        let foundation = seed_foundation(&pool).await;

        let product = sample_product(category.id, material.id, foundation.id);

        let created = repo.create(&product).await.unwrap();
        assert_eq!(created.name, "Product 1");

        let found = repo.find_by_id(product.id).await.unwrap();
        assert_eq!(found.id, product.id);
        assert_eq!(found.category_ids.len(), 1);
        assert_eq!(found.material_ids.len(), 1);
        assert_eq!(found.foundation_ids.len(), 1);
    }

    #[sqlx::test]
    async fn test_create_without_category_should_fail(pool: PgPool) {
        setup_db(&pool).await;
        let repo = ProductRepositoryImpl::new(pool.clone());

        let mut product = sample_product(Uuid::new_v4(), Uuid::new_v4(), Uuid::new_v4());
        product.category_ids = vec![];

        let result = repo.create(&product).await;
        assert!(matches!(result, Err(AppError::Validation(_))));
    }

    #[sqlx::test]
    async fn test_find_all(pool: PgPool) {
        setup_db(&pool).await;
        let repo = ProductRepositoryImpl::new(pool.clone());

        let category = seed_category(&pool).await;
        let material = seed_material(&pool).await;
        let foundation = seed_foundation(&pool).await;

        for _ in 0..3 {
            let product = sample_product(category.id, material.id, foundation.id);
            repo.create(&product).await.unwrap();
        }

        let query = GetProductsQuery {
            pagination: PaginationQuery {
                page: Some(1),
                search: None,
                limit: Some(10),
                sort: None,
                sort_order: None,
            },
            category_id: None,
            material_id: None,
            foundation_id: None,
        };

        let (items, total) = repo.find_all(&query).await.unwrap();

        assert_eq!(total, 3);
        assert_eq!(items.len(), 3);
    }

    #[sqlx::test]
    async fn test_update(pool: PgPool) {
        setup_db(&pool).await;
        let repo = ProductRepositoryImpl::new(pool.clone());

        let category = seed_category(&pool).await;
        let material = seed_material(&pool).await;
        let foundation = seed_foundation(&pool).await;

        let mut product = sample_product(category.id, material.id, foundation.id);
        repo.create(&product).await.unwrap();

        product.name = "Updated Product".to_string();
        product.updated_at = Utc::now();

        let updated = repo.update(product.id, &product).await.unwrap();
        assert_eq!(updated.name, "Updated Product");
    }

    #[sqlx::test]
    async fn test_delete(pool: PgPool) {
        setup_db(&pool).await;
        let repo = ProductRepositoryImpl::new(pool.clone());

        let category = seed_category(&pool).await;
        let material = seed_material(&pool).await;
        let foundation = seed_foundation(&pool).await;

        let product = sample_product(category.id, material.id, foundation.id);
        repo.create(&product).await.unwrap();

        repo.delete(product.id).await.unwrap();

        let result = repo.find_by_id(product.id).await;
        assert!(matches!(result, Err(AppError::NotFound(_))));
    }
}
