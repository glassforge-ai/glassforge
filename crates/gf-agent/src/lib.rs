//! Agent model, presets, and validation.

pub mod model;
pub mod preset;
pub mod strategy;
pub mod validation;

pub use model::{Agent, NewAgent, UpdateAgent, DEFAULT_MODEL};
pub use preset::{AgentPreset, PresetDefaults};
pub use strategy::{Strategy, StrategySet};
pub use validation::{validate_new_agent, validate_update_agent};
