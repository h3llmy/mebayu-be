-- Create join table for products and categories
CREATE TABLE IF NOT EXISTS product_category_relations (
    product_id UUID NOT NULL REFERENCES products(id) ON DELETE CASCADE,
    category_id UUID NOT NULL REFERENCES product_categories(id) ON DELETE CASCADE,
    PRIMARY KEY (product_id, category_id)
);

-- Create join table for products and materials
CREATE TABLE IF NOT EXISTS product_material_relations (
    product_id UUID NOT NULL REFERENCES products(id) ON DELETE CASCADE,
    material_id UUID NOT NULL REFERENCES product_materials(id) ON DELETE CASCADE,
    PRIMARY KEY (product_id, material_id)
);

-- Migrate existing data
INSERT INTO product_category_relations (product_id, category_id)
SELECT id, category_id FROM products;

INSERT INTO product_material_relations (product_id, material_id)
SELECT id, material_id FROM products WHERE material_id IS NOT NULL;

-- Remove foreign key columns from products table
ALTER TABLE products DROP COLUMN category_id;
ALTER TABLE products DROP COLUMN material_id;
