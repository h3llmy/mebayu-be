-- Add migration script here
CREATE TABLE settings (
    id UUID PRIMARY KEY,
    email VARCHAR(255) NOT NULL,
    whatsapp_number VARCHAR(255) NOT NULL,
    hero_images JSONB NOT NULL DEFAULT '[]',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
