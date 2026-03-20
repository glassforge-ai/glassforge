//! Structured events parsed from Claude CLI stream-json output.

use serde::{Deserialize, Serialize};

/// One stream-json event.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum StreamJsonEvent {
    #[serde(rename = "system")]
    System(#[serde(default)] SystemPayload),

    #[serde(rename = "assistant")]
    Assistant(#[serde(default)] AssistantPayload),

    #[serde(rename = "user")]
    User(#[serde(default)] UserPayload),

    #[serde(rename = "result")]
    Result(#[serde(default)] ResultPayload),

    #[serde(rename = "error")]
    Error(#[serde(default)] ErrorPayload),
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SystemPayload {
    pub subtype: Option<String>,
    pub session_id: Option<String>,
    pub model: Option<String>,
    #[serde(default)]
    pub tools: Vec<serde_json::Value>,
    #[serde(default)]
    pub mcp_servers: Vec<serde_json::Value>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AssistantPayload {
    pub message: Option<MessagePayload>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UserPayload {
    pub message: Option<MessagePayload>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MessagePayload {
    pub id: Option<String>,
    pub role: Option<String>,
    #[serde(default)]
    pub content: Vec<ContentBlock>,
    pub usage: Option<serde_json::Value>,
    pub stop_reason: Option<String>,
}

/// Content block: text, tool_use, or tool_result.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ContentBlock {
    #[serde(rename = "text")]
    Text {
        #[serde(default)]
        text: String,
    },
    #[serde(rename = "tool_use")]
    ToolUse {
        id: Option<String>,
        name: Option<String>,
        input: Option<serde_json::Value>,
    },
    #[serde(rename = "tool_result")]
    ToolResult {
        tool_use_id: Option<String>,
        #[serde(default)]
        content: String,
        is_error: Option<bool>,
    },
    #[serde(rename = "thinking")]
    Thinking {
        #[serde(default)]
        thinking: String,
    },
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ResultPayload {
    pub subtype: Option<String>,
    pub result: Option<String>,
    pub cost_usd: Option<f64>,
    pub duration_ms: Option<u64>,
    pub session_id: Option<String>,
    pub usage: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ErrorPayload {
    pub message: Option<String>,
    pub code: Option<String>,
}

/// Normalize any backend's raw stream-json events into ForgeEvent variants.
pub fn normalize_to_forge_event(
    _backend: &str,
    raw: &StreamJsonEvent,
    session_id: &gf_core::ids::SessionId,
    _agent_id: &gf_core::ids::AgentId,
) -> Vec<gf_core::events::ForgeEvent> {
    use chrono::Utc;
    use gf_core::events::ForgeEvent;

    match raw {
        StreamJsonEvent::System(_) => vec![],
        StreamJsonEvent::Assistant(p) => {
            let mut events = Vec::new();
            if let Some(ref msg) = p.message {
                for block in &msg.content {
                    if let Some((kind, content)) = content_block_to_output(block) {
                        events.push(ForgeEvent::ProcessOutput {
                            session_id: *session_id,
                            kind,
                            content,
                            timestamp: Utc::now(),
                        });
                    }
                }
            }
            events
        }
        StreamJsonEvent::User(_) => vec![],
        StreamJsonEvent::Result(_) => {
            vec![ForgeEvent::ProcessCompleted {
                session_id: *session_id,
                exit_code: 0,
                timestamp: Utc::now(),
            }]
        }
        StreamJsonEvent::Error(p) => {
            vec![ForgeEvent::ProcessFailed {
                session_id: *session_id,
                error: p.message.clone().unwrap_or_else(|| "unknown error".into()),
                timestamp: Utc::now(),
            }]
        }
    }
}

fn content_block_to_output(
    block: &ContentBlock,
) -> Option<(gf_core::events::OutputKind, String)> {
    use gf_core::events::OutputKind;

    match block {
        ContentBlock::Text { text } if !text.is_empty() => {
            Some((OutputKind::Assistant, text.clone()))
        }
        ContentBlock::Thinking { thinking } if !thinking.is_empty() => {
            Some((OutputKind::Thinking, thinking.clone()))
        }
        ContentBlock::ToolUse { name, input, .. } => {
            let tool_name = name.as_deref().unwrap_or("unknown");
            let input_str = input
                .as_ref()
                .map(|v| serde_json::to_string(v).unwrap_or_default())
                .unwrap_or_default();
            Some((OutputKind::ToolUse, format!("{}({})", tool_name, input_str)))
        }
        ContentBlock::ToolResult {
            content, is_error, ..
        } => {
            let prefix = if *is_error == Some(true) { "[error] " } else { "" };
            Some((
                OutputKind::ToolResult,
                format!("{}{}", prefix, content),
            ))
        }
        _ => None,
    }
}
