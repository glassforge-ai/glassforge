//! Session CRUD repository.

use chrono::{DateTime, Utc};
use gf_core::error::{ForgeError, ForgeResult};
use gf_core::ids::{AgentId, SessionId};
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub id: SessionId,
    pub agent_id: AgentId,
    pub claude_session_id: Option<String>,
    pub directory: String,
    pub status: String,
    #[serde(default)]
    pub cost_usd: f64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct NewSession {
    pub agent_id: AgentId,
    pub directory: String,
    pub claude_session_id: Option<String>,
}

pub struct SessionRepo {
    conn: Arc<Mutex<Connection>>,
}

impl SessionRepo {
    pub fn new(conn: Arc<Mutex<Connection>>) -> Self {
        Self { conn }
    }

    pub fn create(&self, input: &NewSession) -> ForgeResult<Session> {
        let conn = crate::pool::lock_conn(&self.conn)?;
        let id = SessionId::new();
        let now = Utc::now();
        conn.execute(
            "INSERT INTO sessions (id, agent_id, claude_session_id, directory, status, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, 'created', ?5, ?6)",
            rusqlite::params![
                id.0.to_string(),
                input.agent_id.0.to_string(),
                input.claude_session_id,
                input.directory,
                now.to_rfc3339(),
                now.to_rfc3339(),
            ],
        )
        .map_err(|e| ForgeError::Database(Box::new(e)))?;
        drop(conn);
        self.get(&id)
    }

    pub fn get(&self, id: &SessionId) -> ForgeResult<Session> {
        let conn = crate::pool::lock_conn(&self.conn)?;
        let mut stmt = conn
            .prepare(
                "SELECT id, agent_id, claude_session_id, directory, status, cost_usd, created_at, updated_at
                 FROM sessions WHERE id = ?1",
            )
            .map_err(|e| ForgeError::Database(Box::new(e)))?;
        stmt.query_row(rusqlite::params![id.0.to_string()], row_to_session)
            .map_err(|e| match e {
                rusqlite::Error::QueryReturnedNoRows => ForgeError::NotFound { entity: "session", id: id.to_string() },
                other => ForgeError::Database(Box::new(other)),
            })
    }

    pub fn list(&self) -> ForgeResult<Vec<Session>> {
        let conn = crate::pool::lock_conn(&self.conn)?;
        let mut stmt = conn
            .prepare(
                "SELECT id, agent_id, claude_session_id, directory, status, cost_usd, created_at, updated_at
                 FROM sessions ORDER BY created_at DESC",
            )
            .map_err(|e| ForgeError::Database(Box::new(e)))?;
        let sessions: Vec<Session> = stmt
            .query_map([], row_to_session)
            .map_err(|e| ForgeError::Database(Box::new(e)))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| ForgeError::Database(Box::new(e)))?;
        Ok(sessions)
    }

    pub fn update_status(&self, id: &SessionId, status: &str) -> ForgeResult<Session> {
        let conn = crate::pool::lock_conn(&self.conn)?;
        let now = Utc::now();
        let rows = conn
            .execute(
                "UPDATE sessions SET status = ?1, updated_at = ?2 WHERE id = ?3",
                rusqlite::params![status, now.to_rfc3339(), id.0.to_string()],
            )
            .map_err(|e| ForgeError::Database(Box::new(e)))?;
        if rows == 0 {
            return Err(ForgeError::NotFound { entity: "session", id: id.to_string() });
        }
        drop(conn);
        self.get(id)
    }

    pub fn update_claude_session_id(
        &self,
        id: &SessionId,
        claude_session_id: &str,
    ) -> ForgeResult<Session> {
        let conn = crate::pool::lock_conn(&self.conn)?;
        let now = Utc::now();
        let rows = conn
            .execute(
                "UPDATE sessions SET claude_session_id = ?1, updated_at = ?2 WHERE id = ?3",
                rusqlite::params![claude_session_id, now.to_rfc3339(), id.0.to_string()],
            )
            .map_err(|e| ForgeError::Database(Box::new(e)))?;
        if rows == 0 {
            return Err(ForgeError::NotFound { entity: "session", id: id.to_string() });
        }
        drop(conn);
        self.get(id)
    }

    pub fn delete(&self, id: &SessionId) -> ForgeResult<()> {
        let conn = crate::pool::lock_conn(&self.conn)?;
        let rows = conn
            .execute("DELETE FROM sessions WHERE id = ?1", rusqlite::params![id.0.to_string()])
            .map_err(|e| ForgeError::Database(Box::new(e)))?;
        if rows == 0 {
            return Err(ForgeError::NotFound { entity: "session", id: id.to_string() });
        }
        Ok(())
    }
}

fn row_to_session(row: &rusqlite::Row<'_>) -> Result<Session, rusqlite::Error> {
    let id_str: String = row.get(0)?;
    let id = uuid::Uuid::parse_str(&id_str)
        .map_err(|_| rusqlite::Error::InvalidParameterName(id_str.clone()))?;
    let id = SessionId(id);

    let agent_id_str: String = row.get(1)?;
    let agent_id = uuid::Uuid::parse_str(&agent_id_str)
        .map_err(|_| rusqlite::Error::InvalidParameterName(agent_id_str))?;
    let agent_id = AgentId(agent_id);

    let claude_session_id: Option<String> = row.get(2)?;
    let directory: String = row.get(3)?;
    let status: String = row.get(4)?;
    let cost_usd: f64 = row.get(5)?;
    let created_at: String = row.get(6)?;
    let updated_at: String = row.get(7)?;
    let created_at = DateTime::parse_from_rfc3339(&created_at)
        .map_err(|_| rusqlite::Error::InvalidParameterName(created_at.clone()))?
        .with_timezone(&Utc);
    let updated_at = DateTime::parse_from_rfc3339(&updated_at)
        .map_err(|_| rusqlite::Error::InvalidParameterName(updated_at.clone()))?
        .with_timezone(&Utc);

    Ok(Session {
        id,
        agent_id,
        claude_session_id,
        directory,
        status,
        cost_usd,
        created_at,
        updated_at,
    })
}
