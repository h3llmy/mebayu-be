ALTER TABLE products ADD COLUMN material_id UUID REFERENCES product_materials(id);
