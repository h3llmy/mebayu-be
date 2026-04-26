-- Add description column to category translations
ALTER TABLE product_category_translations ADD COLUMN description TEXT NOT NULL DEFAULT '';

-- Add description column to material translations
ALTER TABLE product_material_translations ADD COLUMN description TEXT NOT NULL DEFAULT '';

-- Add description column to foundation translations
ALTER TABLE product_foundation_translations ADD COLUMN description TEXT NOT NULL DEFAULT '';
