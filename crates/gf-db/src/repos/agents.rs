//! Agent CRUD repository.

use chrono::{DateTime, Utc};
use gf_agent::model::{Agent, NewAgent, UpdateAgent, DEFAULT_MODEL};
use gf_agent::preset::AgentPreset;
use gf_agent::validation::{validate_new_agent, validate_update_agent};
use gf_core::error::{ForgeError, ForgeResult};
use gf_core::ids::AgentId;
use rusqlite::Connection;
use std::sync::{Arc, Mutex};

pub struct AgentRepo {
    conn: Arc<Mutex<Connection>>,
}

impl AgentRepo {
    pub fn new(conn: Arc<Mutex<Connection>>) -> Self {
        Self { conn }
    }

    pub fn create(&self, input: &NewAgent) -> ForgeResult<Agent> {
        validate_new_agent(input)?;

        let conn = crate::pool::lock_conn(&self.conn)?;
        let id = AgentId::new();
        let now = Utc::now();
        let model = input.model.as_deref().unwrap_or(DEFAULT_MODEL).to_string();
        let allowed_tools_json = input
            .allowed_tools
            .as_ref()
            .and_then(|t| serde_json::to_string(t).ok());
        let preset_str = input
            .preset
            .as_ref()
            .and_then(|p| serde_json::to_string(p).ok());
        let config_json = input
            .config
            .as_ref()
            .and_then(|c| serde_json::to_string(c).ok());
        let backend_type = input
            .backend_type
            .as_deref()
            .unwrap_or("claude")
            .to_string();

        conn.execute(
            "INSERT INTO agents (id, name, model, system_prompt, allowed_tools, max_turns, use_max, preset, config_json, backend_type, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
            rusqlite::params![
                id.0.to_string(),
                input.name,
                model,
                input.system_prompt,
                allowed_tools_json,
                input.max_turns.map(i64::from),
                input.use_max.unwrap_or(false),
                preset_str,
                config_json,
                backend_type,
                now.to_rfc3339(),
                now.to_rfc3339(),
            ],
        )
        .map_err(|e| ForgeError::Database(Box::new(e)))?;

        drop(conn);
        self.get(&id)
    }

    pub fn get(&self, id: &AgentId) -> ForgeResult<Agent> {
        let conn = crate::pool::lock_conn(&self.conn)?;
        let mut stmt = conn
            .prepare(
                "SELECT id, name, model, system_prompt, allowed_tools, max_turns, use_max, preset, config_json, backend_type, created_at, updated_at
             FROM agents WHERE id = ?1",
            )
            .map_err(|e| ForgeError::Database(Box::new(e)))?;

        stmt.query_row(
            rusqlite::params![id.0.to_string()],
            |row| row_to_agent(row).map_err(|e| rusqlite::Error::InvalidParameterName(e.to_string())),
        )
        .map_err(|e| match e {
            rusqlite::Error::QueryReturnedNoRows => ForgeError::NotFound { entity: "agent", id: id.to_string() },
            rusqlite::Error::InvalidParameterName(s) => ForgeError::Validation(s),
            other => ForgeError::Database(Box::new(other)),
        })
    }

    pub fn list(&self) -> ForgeResult<Vec<Agent>> {
        let conn = crate::pool::lock_conn(&self.conn)?;
        let mut stmt = conn
            .prepare(
                "SELECT id, name, model, system_prompt, allowed_tools, max_turns, use_max, preset, config_json, backend_type, created_at, updated_at
             FROM agents ORDER BY created_at DESC",
            )
            .map_err(|e| ForgeError::Database(Box::new(e)))?;

        let agents: Vec<Agent> = stmt
            .query_map([], |row| {
                row_to_agent(row).map_err(|e| rusqlite::Error::InvalidParameterName(e.to_string()))
            })
            .map_err(|e| ForgeError::Database(Box::new(e)))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| match e {
                rusqlite::Error::InvalidParameterName(s) => ForgeError::Validation(s),
                other => ForgeError::Database(Box::new(other)),
            })?;

        Ok(agents)
    }

    pub fn update(&self, id: &AgentId, input: &UpdateAgent) -> ForgeResult<Agent> {
        validate_update_agent(input)?;
        let existing = self.get(id)?;
        let now = Utc::now();

        let name = input.name.as_ref().unwrap_or(&existing.name).clone();
        let model = input.model.as_ref().cloned().unwrap_or(existing.model);
        let system_prompt = match &input.system_prompt {
            Some(inner) => inner.clone(),
            None => existing.system_prompt,
        };
        let allowed_tools = match &input.allowed_tools {
            Some(inner) => inner.clone(),
            None => existing.allowed_tools,
        };
        let max_turns = match &input.max_turns {
            Some(inner) => *inner,
            None => existing.max_turns,
        };
        let use_max = input.use_max.unwrap_or(existing.use_max);
        let preset = match &input.preset {
            Some(inner) => inner.clone(),
            None => existing.preset,
        };
        let config = match &input.config {
            Some(inner) => inner.clone(),
            None => existing.config,
        };
        let backend_type = input.backend_type.as_ref().cloned().unwrap_or(existing.backend_type);

        let allowed_tools_json = allowed_tools.as_ref().and_then(|t| serde_json::to_string(t).ok());
        let preset_str = preset.as_ref().and_then(|p| serde_json::to_string(p).ok());
        let config_json = config.as_ref().and_then(|c| serde_json::to_string(c).ok());

        let conn = crate::pool::lock_conn(&self.conn)?;
        conn.execute(
            "UPDATE agents SET name = ?1, model = ?2, system_prompt = ?3, allowed_tools = ?4, max_turns = ?5, use_max = ?6, preset = ?7, config_json = ?8, backend_type = ?9, updated_at = ?10 WHERE id = ?11",
            rusqlite::params![
                name, model, system_prompt, allowed_tools_json,
                max_turns.map(i64::from), use_max, preset_str, config_json,
                backend_type, now.to_rfc3339(), id.0.to_string(),
            ],
        )
        .map_err(|e| ForgeError::Database(Box::new(e)))?;

        drop(conn);
        self.get(id)
    }

    pub fn delete(&self, id: &AgentId) -> ForgeResult<()> {
        let conn = crate::pool::lock_conn(&self.conn)?;
        let rows = conn
            .execute("DELETE FROM agents WHERE id = ?1", rusqlite::params![id.0.to_string()])
            .map_err(|e| ForgeError::Database(Box::new(e)))?;

        if rows == 0 {
            return Err(ForgeError::NotFound { entity: "agent", id: id.to_string() });
        }

        Ok(())
    }
}

fn row_to_agent(row: &rusqlite::Row<'_>) -> Result<Agent, ForgeError> {
    let id_str: String = row.get(0).map_err(|e| ForgeError::Database(Box::new(e)))?;
    let id = uuid::Uuid::parse_str(&id_str)
        .map_err(|_| ForgeError::Validation(format!("invalid agent id: {}", id_str)))?;
    let id = AgentId(id);

    let name: String = row.get(1).map_err(|e| ForgeError::Database(Box::new(e)))?;
    let model: String = row.get(2).map_err(|e| ForgeError::Database(Box::new(e)))?;
    let system_prompt: Option<String> = row.get(3).map_err(|e| ForgeError::Database(Box::new(e)))?;
    let allowed_tools: Option<String> = row.get(4).map_err(|e| ForgeError::Database(Box::new(e)))?;
    let allowed_tools = allowed_tools.as_deref().and_then(|s| serde_json::from_str(s).ok());
    let max_turns: Option<i64> = row.get(5).map_err(|e| ForgeError::Database(Box::new(e)))?;
    let max_turns = max_turns.and_then(|n| u32::try_from(n).ok());
    let use_max: bool = row.get(6).map_err(|e| ForgeError::Database(Box::new(e)))?;
    let preset: Option<String> = row.get(7).map_err(|e| ForgeError::Database(Box::new(e)))?;
    let preset: Option<AgentPreset> = preset
        .as_deref()
        .and_then(|s| serde_json::from_str(s).ok().or_else(|| serde_json::from_str(&format!("\"{}\"", s)).ok()));
    let config_json: Option<String> = row.get(8).map_err(|e| ForgeError::Database(Box::new(e)))?;
    let config = config_json.as_deref().and_then(|s| serde_json::from_str(s).ok());
    let backend_type: String = row.get(9).map_err(|e| ForgeError::Database(Box::new(e)))?;
    let created_at: String = row.get(10).map_err(|e| ForgeError::Database(Box::new(e)))?;
    let updated_at: String = row.get(11).map_err(|e| ForgeError::Database(Box::new(e)))?;
    let created_at = DateTime::parse_from_rfc3339(&created_at)
        .map_err(|_| ForgeError::Validation(format!("invalid timestamp: {}", created_at)))?
        .with_timezone(&Utc);
    let updated_at = DateTime::parse_from_rfc3339(&updated_at)
        .map_err(|_| ForgeError::Validation(format!("invalid timestamp: {}", updated_at)))?
        .with_timezone(&Utc);

    Ok(Agent {
        id,
        name,
        model,
        system_prompt,
        allowed_tools,
        max_turns,
        use_max,
        preset,
        config,
        backend_type,
        created_at,
        updated_at,
    })
}
