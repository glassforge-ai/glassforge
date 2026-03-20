//! Skills repository (stub with essential types).

use gf_core::error::{ForgeError, ForgeResult};
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Skill {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub category: Option<String>,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillRule {
    pub id: String,
    pub skill_id: String,
    pub trigger_type: String,
    pub trigger_pattern: String,
    pub enabled: bool,
}

pub struct SkillRepo {
    conn: Arc<Mutex<Connection>>,
}

impl SkillRepo {
    pub fn new(conn: Arc<Mutex<Connection>>) -> Self {
        Self { conn }
    }

    pub fn list(&self) -> ForgeResult<Vec<Skill>> {
        let conn = crate::pool::lock_conn(&self.conn)?;
        let mut stmt = conn
            .prepare("SELECT id, name, description, category, content FROM skills ORDER BY name")
            .map_err(|e| ForgeError::Database(Box::new(e)))?;
        let skills = stmt
            .query_map([], |row| {
                Ok(Skill {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    description: row.get(2)?,
                    category: row.get(3)?,
                    content: row.get(4)?,
                })
            })
            .map_err(|e| ForgeError::Database(Box::new(e)))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| ForgeError::Database(Box::new(e)))?;
        Ok(skills)
    }
}
