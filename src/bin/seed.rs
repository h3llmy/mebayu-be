use sqlx::postgres::PgPoolOptions;
use std::env;
use uuid::Uuid;
use chrono::Utc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();
    
    let db_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    
    println!("Connecting to database...");
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&db_url)
        .await?;

    println!("Connected! Starting data seed...");

    // 1. Language
    sqlx::query!(
        "INSERT INTO languages (id, code, name, is_default) VALUES ($1, 'en', 'English', true) ON CONFLICT (code) DO NOTHING",
        Uuid::new_v4()
    ).execute(&pool).await?;

    sqlx::query!(
        "INSERT INTO languages (id, code, name, is_default) VALUES ($1, 'id', 'Indonesian', false) ON CONFLICT (code) DO NOTHING",
        Uuid::new_v4()
    ).execute(&pool).await?;

    let en_id = sqlx::query!("SELECT id FROM languages WHERE code = 'en'").fetch_one(&pool).await?.id;
    let id_id = sqlx::query!("SELECT id FROM languages WHERE code = 'id'").fetch_one(&pool).await?.id;

    println!("Languages seeded: English ({}) and Indonesian ({})", en_id, id_id);

    // 2. Clear old data
    println!("Clearing existing test mappings (relations)...");
    sqlx::query!("DELETE FROM product_category_relations").execute(&pool).await?;
    sqlx::query!("DELETE FROM product_material_relations").execute(&pool).await?;
    sqlx::query!("DELETE FROM product_foundation_relations").execute(&pool).await?;
    sqlx::query!("DELETE FROM product_images").execute(&pool).await?;
    sqlx::query!("DELETE FROM product_translations").execute(&pool).await?;
    sqlx::query!("DELETE FROM product_category_translations").execute(&pool).await?;
    sqlx::query!("DELETE FROM product_material_translations").execute(&pool).await?;
    sqlx::query!("DELETE FROM product_foundation_translations").execute(&pool).await?;
    sqlx::query!("DELETE FROM products").execute(&pool).await?;
    sqlx::query!("DELETE FROM product_categories").execute(&pool).await?;
    sqlx::query!("DELETE FROM product_materials").execute(&pool).await?;
    sqlx::query!("DELETE FROM product_foundations").execute(&pool).await?;

    println!("Generating new entities...");

    // 3. Category
    let cat_id = Uuid::new_v4();
    let now = Utc::now();
    sqlx::query!("INSERT INTO product_categories (id, created_at, updated_at) VALUES ($1, $2, $3)", cat_id, now, now).execute(&pool).await?;
    sqlx::query!("INSERT INTO product_category_translations (category_id, language_id, name, description) VALUES ($1, $2, 'Necklaces', 'Elegant necklaces for all occasions.')", cat_id, en_id).execute(&pool).await?;
    sqlx::query!("INSERT INTO product_category_translations (category_id, language_id, name, description) VALUES ($1, $2, 'Kalung', 'Kalung elegan untuk berbagai acara.')", cat_id, id_id).execute(&pool).await?;

    // 4. Material
    let mat_id = Uuid::new_v4();
    sqlx::query!("INSERT INTO product_materials (id, created_at, updated_at) VALUES ($1, $2, $3)", mat_id, now, now).execute(&pool).await?;
    sqlx::query!("INSERT INTO product_material_translations (material_id, language_id, name, description) VALUES ($1, $2, 'Gold 18k', 'High-quality 18k gold.')", mat_id, en_id).execute(&pool).await?;
    sqlx::query!("INSERT INTO product_material_translations (material_id, language_id, name, description) VALUES ($1, $2, 'Emas 18k', 'Emas 18k berkualitas tinggi.')", mat_id, id_id).execute(&pool).await?;

    // 5. Foundation
    let found_id = Uuid::new_v4();
    sqlx::query!("INSERT INTO product_foundations (id, created_at, updated_at) VALUES ($1, $2, $3)", found_id, now, now).execute(&pool).await?;
    sqlx::query!("INSERT INTO product_foundation_translations (foundation_id, language_id, name, description) VALUES ($1, $2, 'Solid', 'Strong and durable foundation.')", found_id, en_id).execute(&pool).await?;
    sqlx::query!("INSERT INTO product_foundation_translations (foundation_id, language_id, name, description) VALUES ($1, $2, 'Padat', 'Pondasi yang kuat dan tahan lama.')", found_id, id_id).execute(&pool).await?;

    // 6. Product
    let prod_id = Uuid::new_v4();
    sqlx::query!("INSERT INTO products (id, price, status, created_at, updated_at) VALUES ($1, 299.99, 'published', $2, $3)", prod_id, now, now).execute(&pool).await?;
    sqlx::query!("INSERT INTO product_translations (product_id, language_id, name, description) VALUES ($1, $2, 'Elara Gold Chain', 'A beautiful 18k gold chain.')", prod_id, en_id).execute(&pool).await?;
    sqlx::query!("INSERT INTO product_translations (product_id, language_id, name, description) VALUES ($1, $2, 'Rantai Emas Elara', 'Rantai emas 18k yang indah.')", prod_id, id_id).execute(&pool).await?;

    // 7. Relations
    sqlx::query!("INSERT INTO product_category_relations (product_id, category_id) VALUES ($1, $2)", prod_id, cat_id).execute(&pool).await?;
    sqlx::query!("INSERT INTO product_material_relations (product_id, material_id) VALUES ($1, $2)", prod_id, mat_id).execute(&pool).await?;
    sqlx::query!("INSERT INTO product_foundation_relations (product_id, foundation_id) VALUES ($1, $2)", prod_id, found_id).execute(&pool).await?;


    // 8. Product Image
    sqlx::query!("INSERT INTO product_images (id, product_id, url, created_at, updated_at) VALUES ($1, $2, 'https://s3.dwikihome.my.id/mebayu/products/9750b53f-c901-4c7a-b2bb-cca4d986d091.jpg', $3, $4)", Uuid::new_v4(), prod_id, now, now).execute(&pool).await?;

    // 9. Settings
    let setting_id = Uuid::new_v4();
    sqlx::query!("DELETE FROM hero_images").execute(&pool).await?;
    sqlx::query!("DELETE FROM settings").execute(&pool).await?;

    sqlx::query!(
        "INSERT INTO settings (id, email, whatsapp_number, created_at, updated_at) VALUES ($1, 'contact@mebayu.com', '+1234567890', $2, $3)",
        setting_id, now, now
    ).execute(&pool).await?;

    sqlx::query!(
        "INSERT INTO hero_images (id, setting_id, image_url, order_index, created_at, updated_at) VALUES ($1, $2, 'https://s3.dwikihome.my.id/mebayu/products/9750b53f-c901-4c7a-b2bb-cca4d986d091.jpg', 0, $3, $4)",
        Uuid::new_v4(), setting_id, now, now
    ).execute(&pool).await?;

    println!("Seeding complete!");

    Ok(())
}
