ALTER TABLE agents ADD COLUMN persona_id TEXT REFERENCES personas(id) ON DELETE SET NULL;
INSERT OR IGNORE INTO schema_version (version) VALUES (12);
