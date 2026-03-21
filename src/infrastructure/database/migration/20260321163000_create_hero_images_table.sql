-- Create hero_images table
CREATE TABLE hero_images (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    setting_id UUID NOT NULL REFERENCES settings(id) ON DELETE CASCADE,
    image_url TEXT NOT NULL,
    order_index INT NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Remove hero_images JSONB column from settings
ALTER TABLE settings DROP COLUMN hero_images;
