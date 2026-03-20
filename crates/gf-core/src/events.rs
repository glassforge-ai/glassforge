//! ForgeEvent enum and OutputKind for system-wide event streaming.
//!
//! Serde format: `#[serde(tag = "type", content = "data")]` so JSON is
//! `{"type": "VariantName", "data": {...}}`.

use crate::ids::{AgentId, SessionId, WorkflowId};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum ForgeEvent {
    // System lifecycle
    SystemStarted {
        version: String,
        timestamp: DateTime<Utc>,
    },
    SystemStopped {
        timestamp: DateTime<Utc>,
    },
    Heartbeat {
        timestamp: DateTime<Utc>,
    },

    // Agent lifecycle
    AgentCreated {
        agent_id: AgentId,
        name: String,
        timestamp: DateTime<Utc>,
    },
    AgentUpdated {
        agent_id: AgentId,
        name: String,
        timestamp: DateTime<Utc>,
    },
    AgentDeleted {
        agent_id: AgentId,
        timestamp: DateTime<Utc>,
    },

    // Process lifecycle
    ProcessStarted {
        session_id: SessionId,
        agent_id: AgentId,
        timestamp: DateTime<Utc>,
    },
    ProcessOutput {
        session_id: SessionId,
        kind: OutputKind,
        content: String,
        timestamp: DateTime<Utc>,
    },
    ProcessCompleted {
        session_id: SessionId,
        exit_code: i32,
        timestamp: DateTime<Utc>,
    },
    ProcessFailed {
        session_id: SessionId,
        error: String,
        timestamp: DateTime<Utc>,
    },

    // Session lifecycle
    SessionCreated {
        session_id: SessionId,
        agent_id: AgentId,
        directory: String,
        timestamp: DateTime<Utc>,
    },
    SessionResumed {
        session_id: SessionId,
        timestamp: DateTime<Utc>,
    },

    // Workflow lifecycle
    WorkflowStarted {
        workflow_id: WorkflowId,
        timestamp: DateTime<Utc>,
    },
    WorkflowStepCompleted {
        workflow_id: WorkflowId,
        step: u32,
        timestamp: DateTime<Utc>,
    },
    WorkflowCompleted {
        workflow_id: WorkflowId,
        timestamp: DateTime<Utc>,
    },
    WorkflowFailed {
        workflow_id: WorkflowId,
        error: String,
        timestamp: DateTime<Utc>,
    },

    // Safety events
    CircuitBreakerTripped {
        agent_id: AgentId,
        reason: String,
        timestamp: DateTime<Utc>,
    },
    BudgetWarning {
        current_cost: f64,
        limit: f64,
        timestamp: DateTime<Utc>,
    },
    BudgetExceeded {
        current_cost: f64,
        limit: f64,
        timestamp: DateTime<Utc>,
    },

    // Hook lifecycle
    HookStarted {
        hook_id: String,
        hook_name: String,
        event_type: String,
        timestamp: DateTime<Utc>,
    },
    HookCompleted {
        hook_id: String,
        hook_name: String,
        duration_ms: u64,
        timestamp: DateTime<Utc>,
    },
    HookFailed {
        hook_id: String,
        hook_name: String,
        error: String,
        timestamp: DateTime<Utc>,
    },

    // Sub-agent lifecycle
    SubAgentRequested {
        parent_session_id: SessionId,
        sub_agent_id: AgentId,
        prompt: String,
        timestamp: DateTime<Utc>,
    },
    SubAgentStarted {
        parent_session_id: SessionId,
        sub_agent_id: AgentId,
        session_id: SessionId,
        timestamp: DateTime<Utc>,
    },
    SubAgentCompleted {
        parent_session_id: SessionId,
        sub_agent_id: AgentId,
        session_id: SessionId,
        timestamp: DateTime<Utc>,
    },
    SubAgentFailed {
        parent_session_id: SessionId,
        sub_agent_id: AgentId,
        error: String,
        timestamp: DateTime<Utc>,
    },

    // Schedule lifecycle
    ScheduleCreated {
        schedule_id: String,
        name: String,
        timestamp: DateTime<Utc>,
    },
    ScheduleTriggered {
        schedule_id: String,
        name: String,
        session_id: SessionId,
        timestamp: DateTime<Utc>,
    },
    ScheduleDeleted {
        schedule_id: String,
        timestamp: DateTime<Utc>,
    },

    // Exit gate
    ExitGateTriggered {
        session_id: SessionId,
        reason: String,
        timestamp: DateTime<Utc>,
    },

    // Quality checks
    QualityCheckStarted {
        session_id: SessionId,
        iteration: u32,
        timestamp: DateTime<Utc>,
    },
    QualityCheckPassed {
        session_id: SessionId,
        score: f64,
        timestamp: DateTime<Utc>,
    },
    QualityCheckFailed {
        session_id: SessionId,
        score: f64,
        reason: String,
        timestamp: DateTime<Utc>,
    },

    // Pipeline lifecycle
    PipelineStarted {
        session_id: String,
        workflow_id: String,
        step_count: usize,
        timestamp: DateTime<Utc>,
    },
    PipelineStepCompleted {
        session_id: String,
        step_index: usize,
        success: bool,
        timestamp: DateTime<Utc>,
    },
    PipelineCompleted {
        session_id: String,
        workflow_id: String,
        success: bool,
        timestamp: DateTime<Utc>,
    },

    // Compaction lifecycle
    CompactionCompleted {
        session_id: String,
        original_tokens: i64,
        compacted_tokens: i64,
        timestamp: DateTime<Utc>,
    },

    // Tool use lifecycle
    ToolUseRequested {
        session_id: SessionId,
        tool_name: String,
        timestamp: DateTime<Utc>,
    },
    ToolUseCompleted {
        session_id: SessionId,
        tool_name: String,
        timestamp: DateTime<Utc>,
    },

    // Session completion
    SessionCompleted {
        session_id: SessionId,
        exit_code: i32,
        timestamp: DateTime<Utc>,
    },

    // Security scan events
    SecurityScanPassed {
        session_id: SessionId,
        timestamp: DateTime<Utc>,
    },
    SecurityScanFailed {
        session_id: SessionId,
        findings: Vec<String>,
        timestamp: DateTime<Utc>,
    },

    // Generic error
    Error {
        message: String,
        context: Option<String>,
        timestamp: DateTime<Utc>,
    },
}

impl ForgeEvent {
    /// Returns the event's embedded timestamp.
    pub fn timestamp(&self) -> DateTime<Utc> {
        match self {
            ForgeEvent::SystemStarted { timestamp, .. }
            | ForgeEvent::SystemStopped { timestamp }
            | ForgeEvent::Heartbeat { timestamp }
            | ForgeEvent::AgentCreated { timestamp, .. }
            | ForgeEvent::AgentUpdated { timestamp, .. }
            | ForgeEvent::AgentDeleted { timestamp, .. }
            | ForgeEvent::ProcessStarted { timestamp, .. }
            | ForgeEvent::ProcessOutput { timestamp, .. }
            | ForgeEvent::ProcessCompleted { timestamp, .. }
            | ForgeEvent::ProcessFailed { timestamp, .. }
            | ForgeEvent::SessionCreated { timestamp, .. }
            | ForgeEvent::SessionResumed { timestamp, .. }
            | ForgeEvent::WorkflowStarted { timestamp, .. }
            | ForgeEvent::WorkflowStepCompleted { timestamp, .. }
            | ForgeEvent::WorkflowCompleted { timestamp, .. }
            | ForgeEvent::WorkflowFailed { timestamp, .. }
            | ForgeEvent::CircuitBreakerTripped { timestamp, .. }
            | ForgeEvent::BudgetWarning { timestamp, .. }
            | ForgeEvent::BudgetExceeded { timestamp, .. }
            | ForgeEvent::HookStarted { timestamp, .. }
            | ForgeEvent::HookCompleted { timestamp, .. }
            | ForgeEvent::HookFailed { timestamp, .. }
            | ForgeEvent::SubAgentRequested { timestamp, .. }
            | ForgeEvent::SubAgentStarted { timestamp, .. }
            | ForgeEvent::SubAgentCompleted { timestamp, .. }
            | ForgeEvent::SubAgentFailed { timestamp, .. }
            | ForgeEvent::ScheduleCreated { timestamp, .. }
            | ForgeEvent::ScheduleTriggered { timestamp, .. }
            | ForgeEvent::ScheduleDeleted { timestamp, .. }
            | ForgeEvent::ExitGateTriggered { timestamp, .. }
            | ForgeEvent::QualityCheckStarted { timestamp, .. }
            | ForgeEvent::QualityCheckPassed { timestamp, .. }
            | ForgeEvent::QualityCheckFailed { timestamp, .. }
            | ForgeEvent::PipelineStarted { timestamp, .. }
            | ForgeEvent::PipelineStepCompleted { timestamp, .. }
            | ForgeEvent::PipelineCompleted { timestamp, .. }
            | ForgeEvent::CompactionCompleted { timestamp, .. }
            | ForgeEvent::ToolUseRequested { timestamp, .. }
            | ForgeEvent::ToolUseCompleted { timestamp, .. }
            | ForgeEvent::SessionCompleted { timestamp, .. }
            | ForgeEvent::SecurityScanPassed { timestamp, .. }
            | ForgeEvent::SecurityScanFailed { timestamp, .. }
            | ForgeEvent::Error { timestamp, .. } => *timestamp,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OutputKind {
    Assistant,
    ToolUse,
    ToolResult,
    Thinking,
    Result,
}
