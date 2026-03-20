ALTER TABLE agents ADD COLUMN backend_type TEXT NOT NULL DEFAULT 'claude';
INSERT INTO schema_version (version) VALUES (14);
