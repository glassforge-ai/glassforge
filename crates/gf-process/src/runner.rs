//! Process runner: maps stream-json events to ForgeEvent and emits to EventBus.

use chrono::Utc;
use gf_core::event_bus::EventBus;
use gf_core::events::{ForgeEvent, OutputKind};
use gf_core::ids::{AgentId, SessionId};
use gf_core::ForgeResult;
use std::sync::Arc;
use tracing::debug;

use crate::stream_event::{ContentBlock, StreamJsonEvent as ParsedEvent};

/// Kind of event from stream-json.
#[derive(Debug, Clone)]
pub enum StreamJsonKind {
    Started,
    Output { kind: OutputKind, content: String },
    Completed { exit_code: i32 },
    Failed { error: String },
}

/// Parsed event from stream-json output.
#[derive(Debug, Clone)]
pub struct StreamJsonEvent {
    pub kind: StreamJsonKind,
}

impl StreamJsonEvent {
    pub fn started() -> Self {
        Self { kind: StreamJsonKind::Started }
    }

    pub fn output(kind: OutputKind, content: impl Into<String>) -> Self {
        Self {
            kind: StreamJsonKind::Output {
                kind,
                content: content.into(),
            },
        }
    }

    pub fn completed(exit_code: i32) -> Self {
        Self { kind: StreamJsonKind::Completed { exit_code } }
    }

    pub fn failed(error: impl Into<String>) -> Self {
        Self {
            kind: StreamJsonKind::Failed { error: error.into() },
        }
    }
}

/// Runs a process and emits process lifecycle events to the EventBus.
pub struct ProcessRunner {
    event_bus: Arc<EventBus>,
}

impl ProcessRunner {
    pub fn new(event_bus: Arc<EventBus>) -> Self {
        Self { event_bus }
    }

    pub fn emit(&self, event: ForgeEvent) -> ForgeResult<()> {
        self.event_bus.emit_sync(event)
    }

    pub fn emit_stub_run(
        &self,
        session_id: SessionId,
        agent_id: AgentId,
    ) -> ForgeResult<()> {
        let now = Utc::now();
        self.event_bus.emit_sync(ForgeEvent::ProcessStarted {
            session_id,
            agent_id,
            timestamp: now,
        })?;
        self.event_bus.emit_sync(ForgeEvent::ProcessOutput {
            session_id,
            kind: OutputKind::Assistant,
            content: "Stub output line 1.".into(),
            timestamp: Utc::now(),
        })?;
        self.event_bus.emit_sync(ForgeEvent::ProcessOutput {
            session_id,
            kind: OutputKind::Assistant,
            content: "Stub output line 2.".into(),
            timestamp: Utc::now(),
        })?;
        self.event_bus.emit_sync(ForgeEvent::ProcessCompleted {
            session_id,
            exit_code: 0,
            timestamp: Utc::now(),
        })?;
        debug!("stub run emitted ProcessStarted, 2x ProcessOutput, ProcessCompleted");
        Ok(())
    }

    pub fn emit_stream_event(
        &self,
        session_id: &SessionId,
        agent_id: &AgentId,
        event: &StreamJsonEvent,
    ) -> ForgeResult<()> {
        let now = Utc::now();
        let forge_event = match &event.kind {
            StreamJsonKind::Started => ForgeEvent::ProcessStarted {
                session_id: *session_id,
                agent_id: *agent_id,
                timestamp: now,
            },
            StreamJsonKind::Output { kind, content } => ForgeEvent::ProcessOutput {
                session_id: *session_id,
                kind: kind.clone(),
                content: content.clone(),
                timestamp: now,
            },
            StreamJsonKind::Completed { exit_code } => ForgeEvent::ProcessCompleted {
                session_id: *session_id,
                exit_code: *exit_code,
                timestamp: now,
            },
            StreamJsonKind::Failed { error } => ForgeEvent::ProcessFailed {
                session_id: *session_id,
                error: error.clone(),
                timestamp: now,
            },
        };
        self.event_bus.emit_sync(forge_event)
    }

    pub fn emit_from_stream(
        &self,
        session_id: SessionId,
        agent_id: AgentId,
        events: impl IntoIterator<Item = StreamJsonEvent>,
    ) -> ForgeResult<()> {
        for event in events {
            self.emit_stream_event(&session_id, &agent_id, &event)?;
        }
        Ok(())
    }

    pub fn emit_parsed_event(
        &self,
        session_id: &SessionId,
        _agent_id: &AgentId,
        event: &ParsedEvent,
    ) -> ForgeResult<()> {
        match event {
            ParsedEvent::System(_) => Ok(()),
            ParsedEvent::Assistant(p) => {
                if let Some(ref msg) = p.message {
                    for block in &msg.content {
                        if let Some((kind, content)) = content_block_output(block) {
                            self.event_bus.emit_sync(ForgeEvent::ProcessOutput {
                                session_id: *session_id,
                                kind,
                                content,
                                timestamp: Utc::now(),
                            })?;
                        }
                    }
                }
                Ok(())
            }
            ParsedEvent::User(_) => Ok(()),
            ParsedEvent::Result(_) => self.event_bus.emit_sync(ForgeEvent::ProcessCompleted {
                session_id: *session_id,
                exit_code: 0,
                timestamp: Utc::now(),
            }),
            ParsedEvent::Error(p) => self.event_bus.emit_sync(ForgeEvent::ProcessFailed {
                session_id: *session_id,
                error: p.message.clone().unwrap_or_else(|| "unknown error".into()),
                timestamp: Utc::now(),
            }),
        }
    }
}

fn content_block_output(block: &ContentBlock) -> Option<(OutputKind, String)> {
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
        ContentBlock::ToolResult { content, is_error, .. } => {
            let prefix = if *is_error == Some(true) { "[error] " } else { "" };
            Some((OutputKind::ToolResult, format!("{}{}", prefix, content)))
        }
        _ => None,
    }
}
