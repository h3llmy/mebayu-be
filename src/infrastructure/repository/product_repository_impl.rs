use std::collections::HashMap;

use async_trait::async_trait;
use sqlx::{PgPool, Row};
use uuid::Uuid;

use crate::{
    core::error::AppError,
    domain::{
        product_categories::entity::ProductCategory,
        product_materials::entity::ProductMaterial,
        products::{
            entity::{Product, ProductImage},
            service::ProductRepository,
        },
    },
    shared::dto::pagination::{PaginationQuery, SortOrder},
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
    async fn find_all(&self, query: &PaginationQuery) -> Result<(Vec<Product>, u64), AppError> {
        let limit = query.get_limit() as i64;
        let offset = query.get_offset();

        let search = query.get_search().map(|s| format!("%{}%", s));

        let allowed_sort_fields = ["name", "price", "created_at", "updated_at", "status"];

        let sort_field = query
            .get_sort()
            .filter(|field| allowed_sort_fields.contains(&field.as_str()))
            .unwrap_or_else(|| "created_at".to_string());

        let sort_order = match query.get_sort_order() {
            Some(SortOrder::Asc) => "ASC",
            _ => "DESC",
        };

        let rows = sqlx::query(&format!(
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

            LEFT JOIN product_images pi
                ON p.id = pi.product_id

            {}
            GROUP BY p.id
            ORDER BY p.{} {}
            LIMIT $1 OFFSET $2
            "#,
            if search.is_some() {
                "WHERE p.name ILIKE $3
                   OR p.material ILIKE $3
                   OR p.description ILIKE $3"
            } else {
                ""
            },
            sort_field,
            sort_order
        ))
        .bind(limit)
        .bind(offset)
        .bind(search)
        .fetch_all(&self.pool)
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
                    categories,
                    product_materials: materials,
                    images,
                }
            })
            .collect();

        Ok((products, total))
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Product, AppError> {
        let row = sqlx::query!(
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

        LEFT JOIN product_images pi 
            ON p.id = pi.product_id

        WHERE p.id = $1
        GROUP BY p.id
        "#,
            id
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?
        .ok_or_else(|| AppError::NotFound("Product not found".to_string()))?;

        let categories: Vec<ProductCategory> =
            serde_json::from_value(row.categories).unwrap_or_default();

        let product_materials: Vec<ProductMaterial> =
            serde_json::from_value(row.product_materials).unwrap_or_default();

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
            categories,
            product_materials,
            images,
        })
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

        // 5. Insert image relations
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

        // 4. Update image relations
        // Clear existing
        sqlx::query("DELETE FROM product_images WHERE product_id = $1")
            .bind(id)
            .execute(&mut *tx)
            .await
            .map_err(|e: sqlx::Error| AppError::Database(e.to_string()))?;

        // Add new
        for image in &product.images {
            sqlx::query(
                "INSERT INTO product_images (id, product_id, url, is_primary, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, $6)",
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
