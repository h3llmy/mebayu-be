CREATE TABLE IF NOT EXISTS setting_translations (
    setting_id UUID NOT NULL REFERENCES settings(id) ON DELETE CASCADE,
    language_id UUID NOT NULL REFERENCES languages(id) ON DELETE CASCADE,
    about_title VARCHAR(255) NOT NULL DEFAULT '',
    about_description TEXT NOT NULL DEFAULT '',
    about_image_url VARCHAR(255) NOT NULL DEFAULT '',
    PRIMARY KEY (setting_id, language_id)
);
