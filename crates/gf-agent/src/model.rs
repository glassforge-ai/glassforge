use chrono::{DateTime, Utc};
use gf_core::AgentId;
use serde::{Deserialize, Serialize};

use crate::preset::AgentPreset;

pub const DEFAULT_MODEL: &str = "claude-sonnet-4-20250514";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Agent {
    pub id: AgentId,
    pub name: String,
    pub model: String,
    pub system_prompt: Option<String>,
    pub allowed_tools: Option<Vec<String>>,
    pub max_turns: Option<u32>,
    pub use_max: bool,
    pub preset: Option<AgentPreset>,
    pub config: Option<serde_json::Value>,
    pub backend_type: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewAgent {
    pub name: String,
    pub model: Option<String>,
    pub system_prompt: Option<String>,
    pub allowed_tools: Option<Vec<String>>,
    pub max_turns: Option<u32>,
    pub use_max: Option<bool>,
    pub preset: Option<AgentPreset>,
    pub config: Option<serde_json::Value>,
    pub backend_type: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateAgent {
    pub name: Option<String>,
    pub model: Option<String>,
    pub system_prompt: Option<Option<String>>,
    pub allowed_tools: Option<Option<Vec<String>>>,
    pub max_turns: Option<Option<u32>>,
    pub use_max: Option<bool>,
    pub preset: Option<Option<AgentPreset>>,
    pub config: Option<Option<serde_json::Value>>,
    pub backend_type: Option<String>,
}
