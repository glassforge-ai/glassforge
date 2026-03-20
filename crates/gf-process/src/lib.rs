//! Process spawning for CLI and stream-json parsing.

pub mod backend;
pub mod best_of_n;
pub mod claude_backend;
pub mod concurrent;
pub mod context_pruner;
pub mod loop_detect;
pub mod parse;
pub mod pipeline;
pub mod review;
pub mod runner;
pub mod skill_router;
pub mod spawn;
pub mod stream_event;
pub mod task_type;

pub use best_of_n::{BestOfNRunner, SelectionResult, select_best};
pub use concurrent::{ConcurrentRunner, SubTask, SubTaskResult, aggregate_results};
pub use loop_detect::{ExitGateConfig, LoopDetector, check_completion_patterns, validate_exit};
pub use parse::parse_line;
pub use pipeline::{Pipeline, PipelineRunner, PipelineStep, StepResult};
pub use runner::{ProcessRunner, StreamJsonEvent as RunnerStubEvent, StreamJsonKind};
pub use backend::{BackendCapabilities, BackendHealth, BackendRegistry, BackendSpawnConfig, ProcessBackend};
pub use claude_backend::ClaudeBackend;
pub use spawn::{ProcessHandle, SpawnConfig, SpawnError, SpawnLimiter, spawn};
pub use stream_event::{
    AssistantPayload, ContentBlock, ErrorPayload, MessagePayload, ResultPayload,
    StreamJsonEvent, SystemPayload, UserPayload, normalize_to_forge_event,
};
