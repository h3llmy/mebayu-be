ALTER TABLE setting_translations
ADD COLUMN hero_title VARCHAR(255) NOT NULL DEFAULT '',
ADD COLUMN hero_description TEXT NOT NULL DEFAULT '';
