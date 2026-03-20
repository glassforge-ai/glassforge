//! Event repository: query events by session, type, and count.

use gf_core::error::ForgeResult;
use gf_core::ids::SessionId;
use rusqlite::Connection;
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone, serde::Serialize)]
pub struct StoredEvent {
    pub id: String,
    pub session_id: Option<String>,
    pub agent_id: Option<String>,
    pub event_type: String,
    pub data_json: String,
    pub timestamp: String,
}

pub struct EventRepo {
    conn: Arc<Mutex<Connection>>,
}

impl EventRepo {
    pub fn new(conn: Arc<Mutex<Connection>>) -> Self {
        Self { conn }
    }

    pub fn query_by_session(&self, session_id: &SessionId) -> ForgeResult<Vec<StoredEvent>> {
        let conn = crate::pool::lock_conn(&self.conn)?;
        let mut stmt = conn
            .prepare(
                "SELECT id, session_id, agent_id, event_type, data_json, timestamp
             FROM events WHERE session_id = ?1 ORDER BY timestamp ASC",
            )
            .map_err(|e| gf_core::error::ForgeError::Database(Box::new(e)))?;
        let rows = stmt
            .query_map(rusqlite::params![session_id.0.to_string()], row_to_stored_event)
            .map_err(|e| gf_core::error::ForgeError::Database(Box::new(e)))?;
        let events = rows
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| gf_core::error::ForgeError::Database(Box::new(e)))?;
        Ok(events)
    }

    pub fn query_by_type(&self, event_type: &str, limit: usize) -> ForgeResult<Vec<StoredEvent>> {
        let conn = crate::pool::lock_conn(&self.conn)?;
        let mut stmt = conn
            .prepare(
                "SELECT id, session_id, agent_id, event_type, data_json, timestamp
             FROM events WHERE event_type = ?1 ORDER BY timestamp DESC LIMIT ?2",
            )
            .map_err(|e| gf_core::error::ForgeError::Database(Box::new(e)))?;
        let rows = stmt
            .query_map(rusqlite::params![event_type, limit as i64], row_to_stored_event)
            .map_err(|e| gf_core::error::ForgeError::Database(Box::new(e)))?;
        let events = rows
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| gf_core::error::ForgeError::Database(Box::new(e)))?;
        Ok(events)
    }

    pub fn count(&self) -> ForgeResult<u64> {
        let conn = crate::pool::lock_conn(&self.conn)?;
        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM events", [], |row| row.get(0))
            .map_err(|e| gf_core::error::ForgeError::Database(Box::new(e)))?;
        Ok(count as u64)
    }
}

fn row_to_stored_event(row: &rusqlite::Row<'_>) -> Result<StoredEvent, rusqlite::Error> {
    Ok(StoredEvent {
        id: row.get(0)?,
        session_id: row.get(1)?,
        agent_id: row.get(2)?,
        event_type: row.get(3)?,
        data_json: row.get(4)?,
        timestamp: row.get(5)?,
    })
}
