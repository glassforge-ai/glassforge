ALTER TABLE schedules ADD COLUMN agent_id TEXT REFERENCES agents(id) ON DELETE CASCADE;
ALTER TABLE schedules ADD COLUMN prompt TEXT;
ALTER TABLE schedules ADD COLUMN directory TEXT DEFAULT '.';
CREATE INDEX IF NOT EXISTS idx_sessions_cost ON sessions(cost_usd);
CREATE INDEX IF NOT EXISTS idx_sessions_created_at ON sessions(created_at);
CREATE INDEX IF NOT EXISTS idx_sessions_agent_id ON sessions(agent_id);
INSERT OR IGNORE INTO schema_version (version) VALUES (5);
