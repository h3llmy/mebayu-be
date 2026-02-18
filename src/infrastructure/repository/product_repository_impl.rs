use std::collections::HashMap;

use async_trait::async_trait;
use sqlx::{PgPool, Postgres, QueryBuilder, Row};
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

        let mut qb = QueryBuilder::<Postgres>::new(
            r#"
        SELECT 
            p.*,
            COUNT(*) OVER() as total_count
        FROM products p
        "#,
        );

        // -------------------------
        // SEARCH
        // -------------------------
        if let Some(search) = query.get_search() {
            qb.push(" WHERE (");
            qb.push("p.name ILIKE ");
            qb.push_bind(format!("%{}%", search));
            qb.push(" OR p.material ILIKE ");
            qb.push_bind(format!("%{}%", search));
            qb.push(" OR p.description ILIKE ");
            qb.push_bind(format!("%{}%", search));
            qb.push(")");
        }

        // -------------------------
        // SORT (whitelisted fields)
        // -------------------------
        let allowed_sort_fields = vec!["name", "price", "created_at", "updated_at", "status"];

        let sort_field = query
            .get_sort()
            .filter(|field| allowed_sort_fields.contains(&field.as_str()))
            .unwrap_or_else(|| "created_at".to_string());

        let sort_order = match query.get_sort_order() {
            Some(SortOrder::Asc) => "ASC",
            _ => "DESC",
        };

        qb.push(" ORDER BY p.");
        qb.push(sort_field);
        qb.push(" ");
        qb.push(sort_order);

        // -------------------------
        // PAGINATION
        // -------------------------
        qb.push(" LIMIT ");
        qb.push_bind(limit);
        qb.push(" OFFSET ");
        qb.push_bind(offset);

        let rows = qb
            .build()
            .fetch_all(&self.pool)
            .await
            .map_err(|e: sqlx::Error| AppError::Database(e.to_string()))?;

        if rows.is_empty() {
            return Ok((vec![], 0));
        }

        let total = rows[0].get::<i64, _>("total_count") as u64;
        let product_ids: Vec<Uuid> = rows.iter().map(|r| r.get("id")).collect();

        // Fetch categories for all products
        let categories_rows = sqlx::query!(
            r#"
            SELECT pcr.product_id, pc.* 
            FROM product_category_relations pcr
            JOIN product_categories pc ON pcr.category_id = pc.id
            WHERE pcr.product_id = ANY($1)
            "#,
            &product_ids
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e: sqlx::Error| AppError::Database(e.to_string()))?;

        // Fetch materials for all products
        let materials_rows = sqlx::query!(
            r#"
            SELECT pmr.product_id, pm.*
            FROM product_material_relations pmr
            JOIN product_materials pm ON pmr.material_id = pm.id
            WHERE pmr.product_id = ANY($1)
            "#,
            &product_ids
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e: sqlx::Error| AppError::Database(e.to_string()))?;

        // Fetch images for all products
        let images_rows = sqlx::query(
            r#"
            SELECT *
            FROM product_images
            WHERE product_id = ANY($1)
            "#,
        )
        .bind(&product_ids)
        .fetch_all(&self.pool)
        .await
        .map_err(|e: sqlx::Error| AppError::Database(e.to_string()))?;

        let products = rows
            .into_iter()
            .map(|r| {
                let id: Uuid = r.get("id");

                let categories: Vec<ProductCategory> = categories_rows
                    .iter()
                    .filter(|cr| cr.product_id == id)
                    .map(|cr| ProductCategory {
                        id: cr.id,
                        name: cr.name.clone(),
                        created_at: cr.created_at,
                        updated_at: cr.updated_at,
                    })
                    .collect();

                let product_materials: Vec<ProductMaterial> = materials_rows
                    .iter()
                    .filter(|mr| mr.product_id == id)
                    .map(|mr| ProductMaterial {
                        id: mr.id,
                        name: mr.name.clone(),
                        created_at: mr.created_at,
                        updated_at: mr.updated_at,
                    })
                    .collect();

                let images: Vec<ProductImage> = images_rows
                    .iter()
                    .filter(|ir| ir.get::<Uuid, _>("product_id") == id)
                    .map(|ir| ProductImage {
                        id: ir.get("id"),
                        product_id: ir.get("product_id"),
                        url: ir.get("url"),
                        created_at: ir.get("created_at"),
                        updated_at: ir.get("updated_at"),
                    })
                    .collect();

                Product {
                    id,
                    name: r.get("name"),
                    material: r.get("material"),
                    price: r.get("price"),
                    description: r.get("description"),
                    status: r.get("status"),
                    created_at: r.get("created_at"),
                    updated_at: r.get("updated_at"),
                    category_ids: categories.iter().map(|c| c.id).collect(),
                    material_ids: product_materials.iter().map(|m| m.id).collect(),
                    categories,
                    product_materials,
                    images,
                }
            })
            .collect();

        Ok((products, total))
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Product, AppError> {
        let product_row = sqlx::query!("SELECT * FROM products WHERE id = $1", id)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e: sqlx::Error| AppError::Database(e.to_string()))?
            .ok_or_else(|| AppError::NotFound("Product not found".to_string()))?;

        let categories = sqlx::query_as!(
            ProductCategory,
            r#"
            SELECT pc.* 
            FROM product_category_relations pcr
            JOIN product_categories pc ON pcr.category_id = pc.id
            WHERE pcr.product_id = $1
            "#,
            id
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e: sqlx::Error| AppError::Database(e.to_string()))?;

        let product_materials = sqlx::query_as!(
            ProductMaterial,
            r#"
            SELECT pm.*
            FROM product_material_relations pmr
            JOIN product_materials pm ON pmr.material_id = pm.id
            WHERE pmr.product_id = $1
            "#,
            id
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e: sqlx::Error| AppError::Database(e.to_string()))?;

        let images = sqlx::query_as::<_, ProductImage>(
            r#"
            SELECT *
            FROM product_images
            WHERE product_id = $1
            "#,
        )
        .bind(id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e: sqlx::Error| AppError::Database(e.to_string()))?;

        Ok(Product {
            id: product_row.id,
            name: product_row.name,
            material: product_row.material,
            price: product_row.price,
            description: product_row.description,
            status: product_row.status,
            created_at: product_row.created_at,
            updated_at: product_row.updated_at,
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
            "INSERT INTO products (id, name, material, price, description, status, created_at, updated_at)
             VALUES ($1,$2,$3,$4,$5,$6,$7,$8)",
            product.id,
            product.name,
            product.material,
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
            "UPDATE products SET name = $2, material = $3, price = $4, description = $5, status = $6, updated_at = $7 WHERE id = $1",
            id,
            product.name,
            product.material,
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
