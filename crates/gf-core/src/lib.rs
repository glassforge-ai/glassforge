//! GlassForge core types: IDs, errors, events, and event bus.
//!
//! This crate is the contract for all other crates. Do not duplicate these types elsewhere.

pub mod error;
pub mod event_bus;
pub mod events;
pub mod ids;

pub use error::{ForgeError, ForgeResult};
pub use event_bus::{EventBus, EventSink};
pub use events::{ForgeEvent, OutputKind};
pub use ids::{AgentId, EventId, SessionId, SkillId, WorkflowId};
