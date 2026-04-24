-- Add languages table
CREATE TABLE IF NOT EXISTS languages (
    id UUID PRIMARY KEY,
    code VARCHAR(10) UNIQUE NOT NULL,
    name VARCHAR(50) NOT NULL,
    is_default BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Insert default language (English)
INSERT INTO languages (id, code, name, is_default)
VALUES (gen_random_uuid(), 'en', 'English', TRUE);

-- Products Translations
CREATE TABLE IF NOT EXISTS product_translations (
    product_id UUID NOT NULL REFERENCES products(id) ON DELETE CASCADE,
    language_id UUID NOT NULL REFERENCES languages(id) ON DELETE CASCADE,
    name VARCHAR(255) NOT NULL,
    description TEXT NOT NULL,
    PRIMARY KEY (product_id, language_id)
);

-- Migrate existing product data to translations table (assuming 'en' for now)
INSERT INTO product_translations (product_id, language_id, name, description)
SELECT p.id, l.id, p.name, p.description
FROM products p
CROSS JOIN languages l
WHERE l.code = 'en';

-- Product Categories Translations
CREATE TABLE IF NOT EXISTS product_category_translations (
    category_id UUID NOT NULL REFERENCES product_categories(id) ON DELETE CASCADE,
    language_id UUID NOT NULL REFERENCES languages(id) ON DELETE CASCADE,
    name VARCHAR(255) NOT NULL,
    PRIMARY KEY (category_id, language_id)
);

-- Migrate existing category data
INSERT INTO product_category_translations (category_id, language_id, name)
SELECT pc.id, l.id, pc.name
FROM product_categories pc
CROSS JOIN languages l
WHERE l.code = 'en';

-- Product Materials Translations
CREATE TABLE IF NOT EXISTS product_material_translations (
    material_id UUID NOT NULL REFERENCES product_materials(id) ON DELETE CASCADE,
    language_id UUID NOT NULL REFERENCES languages(id) ON DELETE CASCADE,
    name VARCHAR(255) NOT NULL,
    PRIMARY KEY (material_id, language_id)
);

-- Migrate existing material data
INSERT INTO product_material_translations (material_id, language_id, name)
SELECT pm.id, l.id, pm.name
FROM product_materials pm
CROSS JOIN languages l
WHERE l.code = 'en';

-- Product Foundations Translations
CREATE TABLE IF NOT EXISTS product_foundation_translations (
    foundation_id UUID NOT NULL REFERENCES product_foundations(id) ON DELETE CASCADE,
    language_id UUID NOT NULL REFERENCES languages(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    PRIMARY KEY (foundation_id, language_id)
);

-- Migrate existing foundation data
INSERT INTO product_foundation_translations (foundation_id, language_id, name)
SELECT pf.id, l.id, pf.name
FROM product_foundations pf
CROSS JOIN languages l
WHERE l.code = 'en';

-- Remove the original columns from source tables
ALTER TABLE products DROP COLUMN name, DROP COLUMN description;
ALTER TABLE product_categories DROP COLUMN name;
ALTER TABLE product_materials DROP COLUMN name;
ALTER TABLE product_foundations DROP COLUMN name;
