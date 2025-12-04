-- Add playlist support columns
ALTER TABLE downloads ADD COLUMN is_playlist BOOLEAN NOT NULL DEFAULT 0;
ALTER TABLE downloads ADD COLUMN total_items INTEGER;
ALTER TABLE downloads ADD COLUMN completed_items INTEGER DEFAULT 0;
