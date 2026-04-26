use std::collections::HashMap;


use async_trait::async_trait;

use sqlx::{PgPool, Row};
use uuid::Uuid;

use crate::{
    core::error::AppError,
    domain::{
        product_categories::entity::{ProductCategory, ProductCategoryTranslation},
        product_foundations::entity::{ProductFoundation, ProductFoundationTranslation},
        product_materials::entity::{ProductMaterial, ProductMaterialTranslation},
        products::{
            dto::GetProductsQuery,
            entity::{Product, ProductImage, ProductTranslation},
            service::ProductRepository,
        },
    },
    shared::dto::pagination::SortOrder,
};
use crate::core::monitoring::observe_db;

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

        let allowed_sort_fields = ["price", "created_at", "updated_at", "status"];
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
                "(
                    EXISTS (SELECT 1 FROM product_translations WHERE product_id = p.id AND (name ILIKE ${} OR description ILIKE ${}))
                    OR EXISTS (
                        SELECT 1 FROM product_category_relations pcr 
                        JOIN product_category_translations pct ON pcr.category_id = pct.category_id 
                        WHERE pcr.product_id = p.id AND pct.name ILIKE ${}
                    )
                    OR EXISTS (
                        SELECT 1 FROM product_material_relations pmr 
                        JOIN product_material_translations pmt ON pmr.material_id = pmt.material_id 
                        WHERE pmr.product_id = p.id AND pmt.name ILIKE ${}
                    )
                    OR EXISTS (
                        SELECT 1 FROM product_foundation_relations pfr 
                        JOIN product_foundation_translations pft ON pfr.foundation_id = pft.foundation_id 
                        WHERE pfr.product_id = p.id AND pft.name ILIKE ${}
                    )
                )",
                param_index, param_index, param_index, param_index, param_index
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

        let count_sql = format!(
            "SELECT COUNT(*) FROM products p {}",
            where_clause
        );
        let mut count_query = sqlx::query_scalar::<_, i64>(&count_sql);

        if let Some(s) = &search {
            count_query = count_query.bind(s);
        }
        if let Some(cid) = query.category_id {
            count_query = count_query.bind(cid);
        }
        if let Some(mid) = query.material_id {
            count_query = count_query.bind(mid);
        }
        if let Some(fid) = query.foundation_id {
            count_query = count_query.bind(fid);
        }

        let total = count_query.fetch_one(&self.pool)
            .await
            .map_err(|e| AppError::Database(e.to_string()))? as u64;

        if total == 0 {
            return Ok((vec![], 0));
        }

        let sql = format!(
            r#"
            SELECT p.id, p.price, p.status, p.created_at, p.updated_at
            FROM products p
            {}
            ORDER BY p.{} {}
            LIMIT $1 OFFSET $2
            "#,
            where_clause, sort_field, sort_order
        );

        let mut sql_query = sqlx::query(&sql).bind(limit).bind(offset);

        if let Some(s) = &search {
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

        let rows = sql_query.fetch_all(&self.pool)
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

        let product_ids: Vec<Uuid> = rows.iter().map(|r| r.get("id")).collect();

        // Fetch translations for all products
        let all_translations = sqlx::query_as!(
            ProductTranslation,
            "SELECT * FROM product_translations WHERE product_id = ANY($1)",
            &product_ids
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

        // images
        let all_images = sqlx::query_as!(
            ProductImage,
            "SELECT * FROM product_images WHERE product_id = ANY($1)",
            &product_ids
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

        // categories
        let all_cat_rows = sqlx::query!(
            "SELECT pcr.product_id, pc.id, pc.created_at, pc.updated_at FROM product_categories pc 
             JOIN product_category_relations pcr ON pc.id = pcr.category_id 
             WHERE pcr.product_id = ANY($1)",
            &product_ids
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

        let cat_ids: Vec<Uuid> = all_cat_rows.iter().map(|r| r.id).collect();
        let all_cat_translations = sqlx::query_as!(
            ProductCategoryTranslation,
            "SELECT category_id, language_id, name FROM product_category_translations WHERE category_id = ANY($1)",
            &cat_ids
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

        // materials
        let all_mat_rows = sqlx::query!(
            "SELECT pmr.product_id, pm.id, pm.created_at, pm.updated_at FROM product_materials pm 
             JOIN product_material_relations pmr ON pm.id = pmr.material_id 
             WHERE pmr.product_id = ANY($1)",
            &product_ids
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

        let mat_ids: Vec<Uuid> = all_mat_rows.iter().map(|r| r.id).collect();
        let all_mat_translations = sqlx::query_as!(
            ProductMaterialTranslation,
            "SELECT material_id, language_id, name FROM product_material_translations WHERE material_id = ANY($1)",
            &mat_ids
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

        // foundations
        let all_found_rows = sqlx::query!(
            "SELECT pfr.product_id, pf.id, pf.created_at, pf.updated_at FROM product_foundations pf 
             JOIN product_foundation_relations pfr ON pf.id = pfr.foundation_id 
             WHERE pfr.product_id = ANY($1)",
            &product_ids
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

        let found_ids: Vec<Uuid> = all_found_rows.iter().map(|r| r.id).collect();
        let all_found_translations = sqlx::query_as!(
            ProductFoundationTranslation,
            "SELECT foundation_id, language_id, name FROM product_foundation_translations WHERE foundation_id = ANY($1)",
            &found_ids
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

        // Map them back
        let products = rows.into_iter().map(|r| {
            let pid: Uuid = r.get("id");
            
            let translations: Vec<ProductTranslation> = all_translations.iter()
                .filter(|t| t.product_id == pid).cloned().collect();
            
            let images: Vec<ProductImage> = all_images.iter()
                .filter(|img| img.product_id == pid).cloned().collect();

            let categories: Vec<ProductCategory> = all_cat_rows.iter()
                .filter(|cr| cr.product_id == pid)
                .map(|cr| {
                    let trans: Vec<ProductCategoryTranslation> = all_cat_translations.iter()
                        .filter(|ct| ct.category_id == cr.id).cloned().collect();
                    ProductCategory {
                        id: cr.id,
                        created_at: cr.created_at,
                        updated_at: cr.updated_at,
                        translations: trans,
                    }
                }).collect();

            let materials: Vec<ProductMaterial> = all_mat_rows.iter()
                .filter(|mr| mr.product_id == pid)
                .map(|mr| {
                    let trans: Vec<ProductMaterialTranslation> = all_mat_translations.iter()
                        .filter(|mt| mt.material_id == mr.id).cloned().collect();
                    ProductMaterial {
                        id: mr.id,
                        created_at: mr.created_at,
                        updated_at: mr.updated_at,
                        translations: trans,
                    }
                }).collect();

            let foundations: Vec<ProductFoundation> = all_found_rows.iter()
                .filter(|fr| fr.product_id == pid)
                .map(|fr| {
                    let trans: Vec<ProductFoundationTranslation> = all_found_translations.iter()
                        .filter(|ft| ft.foundation_id == fr.id).cloned().collect();
                    ProductFoundation {
                        id: fr.id,
                        created_at: fr.created_at,
                        updated_at: fr.updated_at,
                        translations: trans,
                    }
                }).collect();

            Product {
                id: pid,
                price: r.get("price"),
                status: r.get("status"),
                created_at: r.get("created_at"),
                updated_at: r.get("updated_at"),
                category_ids: categories.iter().map(|c| c.id).collect(),
                material_ids: materials.iter().map(|m| m.id).collect(),
                foundation_ids: foundations.iter().map(|f| f.id).collect(),
                product_categories: categories,
                product_materials: materials,
                product_foundations: foundations,
                images,
                translations,
            }
        }).collect();

        Ok((products, total))
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Product, AppError> {
        let row = observe_db(
            "product.find_by_id.base",
            sqlx::query!(
                "SELECT id, price, status, created_at, updated_at FROM products WHERE id = $1",
                id
            )
            .fetch_optional(&self.pool),
        )
        .await
        .map_err(|e| AppError::Database(e.to_string()))?
        .ok_or_else(|| AppError::NotFound("Product not found".to_string()))?;

        let translations = sqlx::query_as!(
            ProductTranslation,
            "SELECT * FROM product_translations WHERE product_id = $1",
            id
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

        let images = sqlx::query_as!(
            ProductImage,
            "SELECT * FROM product_images WHERE product_id = $1",
            id
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

        // Categories
        let category_rows = sqlx::query!(
            "SELECT pc.id, pc.created_at, pc.updated_at FROM product_categories pc 
             JOIN product_category_relations pcr ON pc.id = pcr.category_id 
             WHERE pcr.product_id = $1",
            id
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

        let mut categories = Vec::new();
        for cat_row in category_rows {
            let cat_translations = sqlx::query_as!(
                ProductCategoryTranslation,
                "SELECT category_id, language_id, name FROM product_category_translations WHERE category_id = $1",
                cat_row.id
            )
            .fetch_all(&self.pool)
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

            categories.push(ProductCategory {
                id: cat_row.id,
                created_at: cat_row.created_at,
                updated_at: cat_row.updated_at,
                translations: cat_translations,
            });
        }

        // Materials
        let material_rows = sqlx::query!(
            "SELECT pm.id, pm.created_at, pm.updated_at FROM product_materials pm 
             JOIN product_material_relations pmr ON pm.id = pmr.material_id 
             WHERE pmr.product_id = $1",
            id
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

        let mut materials = Vec::new();
        for mat_row in material_rows {
            let mat_translations = sqlx::query_as!(
                ProductMaterialTranslation,
                "SELECT material_id, language_id, name FROM product_material_translations WHERE material_id = $1",
                mat_row.id
            )
            .fetch_all(&self.pool)
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

            materials.push(ProductMaterial {
                id: mat_row.id,
                created_at: mat_row.created_at,
                updated_at: mat_row.updated_at,
                translations: mat_translations,
            });
        }

        // Foundations
        let foundation_rows = sqlx::query!(
            "SELECT pf.id, pf.created_at, pf.updated_at FROM product_foundations pf 
             JOIN product_foundation_relations pfr ON pf.id = pfr.foundation_id 
             WHERE pfr.product_id = $1",
            id
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

        let mut foundations = Vec::new();
        for f_row in foundation_rows {
            let f_translations = sqlx::query_as!(
                ProductFoundationTranslation,
                "SELECT foundation_id, language_id, name FROM product_foundation_translations WHERE foundation_id = $1",
                f_row.id
            )
            .fetch_all(&self.pool)
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

            foundations.push(ProductFoundation {
                id: f_row.id,
                created_at: f_row.created_at,
                updated_at: f_row.updated_at,
                translations: f_translations,
            });
        }

        Ok(Product {
            id: row.id,
            price: row.price,
            status: row.status,
            created_at: row.created_at,
            updated_at: row.updated_at,
            category_ids: categories.iter().map(|c| c.id).collect(),
            material_ids: materials.iter().map(|m| m.id).collect(),
            foundation_ids: foundations.iter().map(|f| f.id).collect(),
            product_categories: categories,
            product_materials: materials,
            product_foundations: foundations,
            images,
            translations,
        })
    }

    async fn find_recommendations(&self, id: Uuid, limit: i64) -> Result<Vec<Product>, AppError> {
        let sql = r#"
            SELECT
                p.id, p.price, p.status, p.created_at, p.updated_at,
                COUNT(DISTINCT pcr2.category_id) + COUNT(DISTINCT pmr2.material_id) + COUNT(DISTINCT pfr2.foundation_id) AS overlap_score
            FROM products p
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

        let product_ids: Vec<Uuid> = rows.iter().map(|r| r.get("id")).collect();
        if product_ids.is_empty() {
            return Ok(vec![]);
        }

        // Reuse the logic from find_all to map product details
        // Translations
        let all_translations = sqlx::query_as!(
            ProductTranslation,
            "SELECT * FROM product_translations WHERE product_id = ANY($1)",
            &product_ids
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

        // images
        let all_images = sqlx::query_as!(
            ProductImage,
            "SELECT * FROM product_images WHERE product_id = ANY($1)",
            &product_ids
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

        // associations
        let all_cat_rows = sqlx::query!(
            "SELECT pcr.product_id, pc.id, pc.created_at, pc.updated_at FROM product_categories pc 
             JOIN product_category_relations pcr ON pc.id = pcr.category_id 
             WHERE pcr.product_id = ANY($1)",
            &product_ids
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

        let cat_ids: Vec<Uuid> = all_cat_rows.iter().map(|cr| cr.id).collect();
        let all_cat_translations = sqlx::query_as!(
            ProductCategoryTranslation,
            "SELECT category_id, language_id, name FROM product_category_translations WHERE category_id = ANY($1)",
            &cat_ids
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

        let all_mat_rows = sqlx::query!(
            "SELECT pmr.product_id, pm.id, pm.created_at, pm.updated_at FROM product_materials pm 
             JOIN product_material_relations pmr ON pm.id = pmr.material_id 
             WHERE pmr.product_id = ANY($1)",
            &product_ids
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

        let mat_ids: Vec<Uuid> = all_mat_rows.iter().map(|mr| mr.id).collect();
        let all_mat_translations = sqlx::query_as!(
            ProductMaterialTranslation,
            "SELECT material_id, language_id, name FROM product_material_translations WHERE material_id = ANY($1)",
            &mat_ids
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

        let all_found_rows = sqlx::query!(
            "SELECT pfr.product_id, pf.id, pf.created_at, pf.updated_at FROM product_foundations pf 
             JOIN product_foundation_relations pfr ON pf.id = pfr.foundation_id 
             WHERE pfr.product_id = ANY($1)",
            &product_ids
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

        let found_ids: Vec<Uuid> = all_found_rows.iter().map(|fr| fr.id).collect();
        let all_found_translations = sqlx::query_as!(
            ProductFoundationTranslation,
            "SELECT foundation_id, language_id, name FROM product_foundation_translations WHERE foundation_id = ANY($1)",
            &found_ids
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

        let products = rows.into_iter().map(|r| {
            let pid: Uuid = r.get("id");
            
            let translations: Vec<ProductTranslation> = all_translations.iter()
                .filter(|t| t.product_id == pid).cloned().collect();
            
            let images: Vec<ProductImage> = all_images.iter()
                .filter(|img| img.product_id == pid).cloned().collect();

            let categories: Vec<ProductCategory> = all_cat_rows.iter()
                .filter(|cr| cr.product_id == pid)
                .map(|cr| {
                    let trans: Vec<ProductCategoryTranslation> = all_cat_translations.iter()
                        .filter(|ct| ct.category_id == cr.id).cloned().collect();
                    ProductCategory {
                        id: cr.id,
                        created_at: cr.created_at,
                        updated_at: cr.updated_at,
                        translations: trans,
                    }
                }).collect();

            let materials: Vec<ProductMaterial> = all_mat_rows.iter()
                .filter(|mr| mr.product_id == pid)
                .map(|mr| {
                    let trans: Vec<ProductMaterialTranslation> = all_mat_translations.iter()
                        .filter(|mt| mt.material_id == mr.id).cloned().collect();
                    ProductMaterial {
                        id: mr.id,
                        created_at: mr.created_at,
                        updated_at: mr.updated_at,
                        translations: trans,
                    }
                }).collect();

            let foundations: Vec<ProductFoundation> = all_found_rows.iter()
                .filter(|fr| fr.product_id == pid)
                .map(|fr| {
                    let trans: Vec<ProductFoundationTranslation> = all_found_translations.iter()
                        .filter(|ft| ft.foundation_id == fr.id).cloned().collect();
                    ProductFoundation {
                        id: fr.id,
                        created_at: fr.created_at,
                        updated_at: fr.updated_at,
                        translations: trans,
                    }
                }).collect();

            Product {
                id: pid,
                price: r.get("price"),
                status: r.get("status"),
                created_at: r.get("created_at"),
                updated_at: r.get("updated_at"),
                category_ids: categories.iter().map(|c| c.id).collect(),
                material_ids: materials.iter().map(|m| m.id).collect(),
                foundation_ids: foundations.iter().map(|f| f.id).collect(),
                product_categories: categories,
                product_materials: materials,
                product_foundations: foundations,
                images,
                translations,
            }
        }).collect();

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
            "INSERT INTO products (id, price, status, created_at, updated_at)
             VALUES ($1,$2,$3,$4,$5)",
            product.id,
            product.price,
            product.status,
            product.created_at,
            product.updated_at
        )
        .execute(&mut *tx)
        .await
        .map_err(|e: sqlx::Error| AppError::Database(e.to_string()))?;

        // 3. Insert translations
        for translation in &product.translations {
            sqlx::query!(
                "INSERT INTO product_translations (product_id, language_id, name, description)
                 VALUES ($1, $2, $3, $4)",
                product.id,
                translation.language_id,
                translation.name,
                translation.description
            )
            .execute(&mut *tx)
            .await
            .map_err(|e: sqlx::Error| AppError::Database(e.to_string()))?;
        }

        // 4. Insert category relations
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

        // 5. Insert material relations
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

        // 6. Insert foundation relations
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

        // 7. Insert images
        for image in &product.images {
            sqlx::query!(
                "INSERT INTO product_images (id, product_id, url, created_at, updated_at) VALUES ($1, $2, $3, $4, $5)",
                image.id,
                product.id,
                image.url,
                image.created_at,
                image.updated_at
            )
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
            "UPDATE products SET price = $2, status = $3, updated_at = $4 WHERE id = $1",
            id,
            product.price,
            product.status,
            product.updated_at
        )
        .execute(&mut *tx)
        .await
        .map_err(|e: sqlx::Error| AppError::Database(e.to_string()))?;

        // 2. Update translations
        sqlx::query!("DELETE FROM product_translations WHERE product_id = $1", id)
            .execute(&mut *tx)
            .await
            .map_err(|e: sqlx::Error| AppError::Database(e.to_string()))?;

        for translation in &product.translations {
            sqlx::query!(
                "INSERT INTO product_translations (product_id, language_id, name, description)
                 VALUES ($1, $2, $3, $4)",
                id,
                translation.language_id,
                translation.name,
                translation.description
            )
            .execute(&mut *tx)
            .await
            .map_err(|e: sqlx::Error| AppError::Database(e.to_string()))?;
        }

        // 3. Update category relations
        sqlx::query!("DELETE FROM product_category_relations WHERE product_id = $1", id)
            .execute(&mut *tx)
            .await
            .map_err(|e: sqlx::Error| AppError::Database(e.to_string()))?;

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

        // 4. Update material relations
        sqlx::query!("DELETE FROM product_material_relations WHERE product_id = $1", id)
            .execute(&mut *tx)
            .await
            .map_err(|e: sqlx::Error| AppError::Database(e.to_string()))?;

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

        // 5. Update foundation relations
        sqlx::query!(
            "DELETE FROM product_foundation_relations WHERE product_id = $1",
            id
        )
        .execute(&mut *tx)
        .await
        .map_err(|e: sqlx::Error| AppError::Database(e.to_string()))?;

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

        // 6. Update images
        sqlx::query!("DELETE FROM product_images WHERE product_id = $1", id)
            .execute(&mut *tx)
            .await
            .map_err(|e: sqlx::Error| AppError::Database(e.to_string()))?;

        for image in &product.images {
            sqlx::query!(
                "INSERT INTO product_images (id, product_id, url, created_at, updated_at) VALUES ($1, $2, $3, $4, $5)",
                image.id,
                id,
                image.url,
                image.created_at,
                image.updated_at
            )
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

    async fn seed_language(pool: &PgPool) -> Uuid {
        let id = Uuid::new_v4();
        sqlx::query!(
            "INSERT INTO languages (id, code, name, is_default) VALUES ($1, $2, $3, $4)",
            id, "en", "English", true
        )
        .execute(pool)
        .await
        .unwrap();
        id
    }

    async fn seed_category(pool: &PgPool, lang_id: Uuid) -> ProductCategory {
        let id = Uuid::new_v4();
        let now = Utc::now();

        sqlx::query!(
            "INSERT INTO product_categories (id, created_at, updated_at) VALUES ($1, $2, $3)",
            id, now, now
        )
        .execute(pool)
        .await
        .unwrap();

        let translation = ProductCategoryTranslation {
            category_id: id,
            language_id: lang_id,
            name: "Category 1".to_string(),
        };

        sqlx::query!(
            "INSERT INTO product_category_translations (category_id, language_id, name) VALUES ($1, $2, $3)",
            id, lang_id, translation.name
        )
        .execute(pool)
        .await
        .unwrap();

        ProductCategory {
            id,
            created_at: now,
            updated_at: now,
            translations: vec![translation],
        }
    }

    async fn seed_material(pool: &PgPool, lang_id: Uuid) -> ProductMaterial {
        let id = Uuid::new_v4();
        let now = Utc::now();

        sqlx::query!(
            "INSERT INTO product_materials (id, created_at, updated_at) VALUES ($1, $2, $3)",
            id, now, now
        )
        .execute(pool)
        .await
        .unwrap();

        let translation = ProductMaterialTranslation {
            material_id: id,
            language_id: lang_id,
            name: "Material 1".to_string(),
        };

        sqlx::query!(
            "INSERT INTO product_material_translations (material_id, language_id, name) VALUES ($1, $2, $3)",
            id, lang_id, translation.name
        )
        .execute(pool)
        .await
        .unwrap();

        ProductMaterial {
            id,
            created_at: now,
            updated_at: now,
            translations: vec![translation],
        }
    }

    async fn seed_foundation(pool: &PgPool, lang_id: Uuid) -> ProductFoundation {
        let id = Uuid::new_v4();
        let now = Utc::now();

        sqlx::query!(
            "INSERT INTO product_foundations (id, created_at, updated_at) VALUES ($1, $2, $3)",
            id, now, now
        )
        .execute(pool)
        .await
        .unwrap();

        let translation = ProductFoundationTranslation {
            foundation_id: id,
            language_id: lang_id,
            name: "Foundation 1".to_string(),
        };

        sqlx::query!(
            "INSERT INTO product_foundation_translations (foundation_id, language_id, name) VALUES ($1, $2, $3)",
            id, lang_id, translation.name
        )
        .execute(pool)
        .await
        .unwrap();

        ProductFoundation {
            id,
            created_at: now,
            updated_at: now,
            translations: vec![translation],
        }
    }

    fn sample_product(lang_id: Uuid, category_id: Uuid, material_id: Uuid, foundation_id: Uuid) -> Product {
        let id = Uuid::new_v4();
        Product {
            id,
            price: 100.0,
            status: "ACTIVE".to_string(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            category_ids: vec![category_id],
            material_ids: vec![material_id],
            foundation_ids: vec![foundation_id],
            product_categories: vec![],
            product_materials: vec![],
            product_foundations: vec![],
            images: vec![],
            translations: vec![ProductTranslation {
                product_id: id,
                language_id: lang_id,
                name: "Product 1".to_string(),
                description: "Test product".to_string(),
            }],
        }
    }

    #[sqlx::test]
    async fn test_create_and_find_by_id(pool: PgPool) {
        setup_db(&pool).await;
        let repo = ProductRepositoryImpl::new(pool.clone());
        let lang_id = seed_language(&pool).await;

        let category = seed_category(&pool, lang_id).await;
        let material = seed_material(&pool, lang_id).await;
        let foundation = seed_foundation(&pool, lang_id).await;

        let product = sample_product(lang_id, category.id, material.id, foundation.id);

        let created = repo.create(&product).await.unwrap();
        assert_eq!(created.translations[0].name, "Product 1");

        let found = repo.find_by_id(product.id).await.unwrap();
        assert_eq!(found.id, product.id);
        assert_eq!(found.category_ids.len(), 1);
        assert_eq!(found.translations.len(), 1);
    }

    #[sqlx::test]
    async fn test_create_without_category_should_fail(pool: PgPool) {
        setup_db(&pool).await;
        let repo = ProductRepositoryImpl::new(pool.clone());
        let lang_id = seed_language(&pool).await;

        let mut product = sample_product(lang_id, Uuid::new_v4(), Uuid::new_v4(), Uuid::new_v4());
        product.category_ids = vec![];

        let result = repo.create(&product).await;
        assert!(matches!(result, Err(AppError::Validation(_))));
    }

    #[sqlx::test]
    async fn test_find_all(pool: PgPool) {
        setup_db(&pool).await;
        let repo = ProductRepositoryImpl::new(pool.clone());
        let lang_id = seed_language(&pool).await;

        let category = seed_category(&pool, lang_id).await;
        let material = seed_material(&pool, lang_id).await;
        let foundation = seed_foundation(&pool, lang_id).await;

        for _ in 0..3 {
            let product = sample_product(lang_id, category.id, material.id, foundation.id);
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
        assert_eq!(items[0].translations[0].name, "Product 1");
    }

    #[sqlx::test]
    async fn test_update(pool: PgPool) {
        setup_db(&pool).await;
        let repo = ProductRepositoryImpl::new(pool.clone());
        let lang_id = seed_language(&pool).await;

        let category = seed_category(&pool, lang_id).await;
        let material = seed_material(&pool, lang_id).await;
        let foundation = seed_foundation(&pool, lang_id).await;

        let mut product = sample_product(lang_id, category.id, material.id, foundation.id);
        repo.create(&product).await.unwrap();

        product.translations[0].name = "Updated Product".to_string();
        product.updated_at = Utc::now();

        let updated = repo.update(product.id, &product).await.unwrap();
        assert_eq!(updated.translations[0].name, "Updated Product");
    }

    #[sqlx::test]
    async fn test_delete(pool: PgPool) {
        setup_db(&pool).await;
        let repo = ProductRepositoryImpl::new(pool.clone());
        let lang_id = seed_language(&pool).await;

        let category = seed_category(&pool, lang_id).await;
        let material = seed_material(&pool, lang_id).await;
        let foundation = seed_foundation(&pool, lang_id).await;

        let product = sample_product(lang_id, category.id, material.id, foundation.id);
        repo.create(&product).await.unwrap();

        repo.delete(product.id).await.unwrap();

        let result = repo.find_by_id(product.id).await;
        assert!(matches!(result, Err(AppError::NotFound(_))));
    }
}
