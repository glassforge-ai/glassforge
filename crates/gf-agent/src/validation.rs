use crate::model::{NewAgent, UpdateAgent};
use gf_core::error::{ForgeError, ForgeResult};

const MAX_NAME_LENGTH: usize = 100;
const MAX_SYSTEM_PROMPT_LENGTH: usize = 102_400; // 100KB

/// Agent name may only contain ASCII alphanumeric, hyphen, and underscore.
fn name_char_set_valid(name: &str) -> bool {
    name.chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
}

pub fn validate_new_agent(agent: &NewAgent) -> ForgeResult<()> {
    if agent.name.trim().is_empty() {
        return Err(ForgeError::Validation("Agent name cannot be empty".into()));
    }
    if !name_char_set_valid(&agent.name) {
        return Err(ForgeError::Validation(
            "Agent name may only contain letters, numbers, hyphens, and underscores".into(),
        ));
    }
    if agent.name.len() > MAX_NAME_LENGTH {
        return Err(ForgeError::Validation(format!(
            "Agent name must be {} characters or fewer",
            MAX_NAME_LENGTH
        )));
    }
    if let Some(ref prompt) = agent.system_prompt {
        if prompt.len() > MAX_SYSTEM_PROMPT_LENGTH {
            return Err(ForgeError::Validation(
                "System prompt must be 100KB or fewer".into(),
            ));
        }
    }
    Ok(())
}

/// Validates an UpdateAgent. If name is Some, it must be non-empty and within length.
pub fn validate_update_agent(agent: &UpdateAgent) -> ForgeResult<()> {
    if let Some(ref name) = agent.name {
        if name.trim().is_empty() {
            return Err(ForgeError::Validation(
                "Agent name cannot be empty".into(),
            ));
        }
        if !name_char_set_valid(name) {
            return Err(ForgeError::Validation(
                "Agent name may only contain letters, numbers, hyphens, and underscores".into(),
            ));
        }
        if name.len() > MAX_NAME_LENGTH {
            return Err(ForgeError::Validation(format!(
                "Agent name must be {} characters or fewer",
                MAX_NAME_LENGTH
            )));
        }
    }
    if let Some(Some(ref prompt)) = agent.system_prompt {
        if prompt.len() > MAX_SYSTEM_PROMPT_LENGTH {
            return Err(ForgeError::Validation(
                "System prompt must be 100KB or fewer".into(),
            ));
        }
    }
    Ok(())
}
