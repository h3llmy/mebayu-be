use async_trait::async_trait;
use sqlx::{PgPool, Postgres, QueryBuilder, Row};
use uuid::Uuid;

use crate::{
    core::error::AppError,
    domain::{
        product_categories::entity::ProductCategory,
        product_materials::entity::ProductMaterial,
        products::{entity::Product, service::ProductRepository},
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
            c.name as category_name,
            c.created_at as category_created_at,
            c.updated_at as category_updated_at,
            m.name as material_name,
            m.created_at as material_created_at,
            m.updated_at as material_updated_at,
            COUNT(*) OVER() as total_count
        FROM products p
        JOIN product_categories c ON p.category_id = c.id
        LEFT JOIN product_materials m ON p.material_id = m.id
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

        let total = rows
            .first()
            .and_then(|r| r.try_get::<i64, _>("total_count").ok())
            .unwrap_or(0);

        let products = rows
            .into_iter()
            .map(|r| {
                let material_id: Option<Uuid> = r.get("material_id");
                let material_name: Option<String> = r.get("material_name");
                let material_created_at: Option<chrono::DateTime<chrono::Utc>> =
                    r.get("material_created_at");
                let material_updated_at: Option<chrono::DateTime<chrono::Utc>> =
                    r.get("material_updated_at");

                let product_material =
                    if let (Some(id), Some(name), Some(created_at), Some(updated_at)) = (
                        material_id,
                        material_name,
                        material_created_at,
                        material_updated_at,
                    ) {
                        Some(ProductMaterial {
                            id,
                            name,
                            created_at,
                            updated_at,
                        })
                    } else {
                        None
                    };

                Product {
                    id: r.get("id"),
                    category_id: r.get("category_id"),
                    material_id,
                    name: r.get("name"),
                    material: r.get("material"),
                    price: r.get("price"),
                    description: r.get("description"),
                    status: r.get("status"),
                    created_at: r.get("created_at"),
                    updated_at: r.get("updated_at"),
                    category: Some(ProductCategory {
                        id: r.get("category_id"),
                        name: r.get("category_name"),
                        created_at: r.get("category_created_at"),
                        updated_at: r.get("category_updated_at"),
                    }),
                    product_material,
                }
            })
            .collect();
        Ok((products, total as u64))
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Product, AppError> {
        let row = sqlx::query!(
            r#"
            SELECT 
                p.*,
                c.name as category_name,
                c.created_at as category_created_at,
                c.updated_at as category_updated_at,
                m.name as "material_name?",
                m.created_at as "material_created_at?",
                m.updated_at as "material_updated_at?"
            FROM products p
            JOIN product_categories c ON p.category_id = c.id
            LEFT JOIN product_materials m ON p.material_id = m.id
            WHERE p.id = $1
            "#,
            id
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|_| AppError::NotFound("Product not found".to_string()))?;

        let product_material = if let (Some(id), Some(name), Some(created_at), Some(updated_at)) = (
            row.material_id,
            row.material_name,
            row.material_created_at,
            row.material_updated_at,
        ) {
            Some(ProductMaterial {
                id,
                name,
                created_at,
                updated_at,
            })
        } else {
            None
        };

        Ok(Product {
            id: row.id,
            category_id: row.category_id,
            material_id: row.material_id,
            name: row.name,
            material: row.material,
            price: row.price,
            description: row.description,
            status: row.status,
            created_at: row.created_at,
            updated_at: row.updated_at,
            category: Some(ProductCategory {
                id: row.category_id,
                name: row.category_name,
                created_at: row.category_created_at,
                updated_at: row.category_updated_at,
            }),
            product_material,
        })
    }

    async fn create(&self, product: &Product) -> Result<Product, AppError> {
        let category_exist = sqlx::query!(
            "SELECT id FROM product_categories WHERE id = $1",
            product.category_id
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e: sqlx::Error| AppError::Database(e.to_string()))?;

        if category_exist.is_none() {
            return Err(AppError::NotFound("Category not found".to_string()));
        }

        let row = sqlx::query!(
            "INSERT INTO products (id, category_id, material_id, name, material, price, description, status, created_at, updated_at)
             VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10) RETURNING id",
            product.id,
            product.category_id,
            product.material_id,
            product.name,
            product.material,
            product.price,
            product.description,
            product.status,
            product.created_at,
            product.updated_at
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e: sqlx::Error| AppError::Database(e.to_string()))?;

        self.find_by_id(row.id).await
    }

    async fn update(&self, id: Uuid, product: &Product) -> Result<Product, AppError> {
        sqlx::query!(
            "UPDATE products SET category_id = $2, material_id = $3, name = $4, material = $5, price = $6, description = $7, status = $8, updated_at = $9 WHERE id = $1",
            id,
            product.category_id,
            product.material_id,
            product.name,
            product.material,
            product.price,
            product.description,
            product.status,
            product.updated_at
        )
        .execute(&self.pool)
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
