//! Parse stream-json lines into StreamJsonEvent.

use crate::stream_event::StreamJsonEvent;
use gf_core::ForgeError;

/// Parse a single line of stream-json output.
pub fn parse_line(line: &str) -> Result<Option<StreamJsonEvent>, ForgeError> {
    let line = line.trim();
    if line.is_empty() {
        return Ok(None);
    }
    let value: serde_json::Value = serde_json::from_str(line)
        .map_err(|e| ForgeError::Process(format!("JSON parse error: {e}")))?;
    let typ = value
        .get("type")
        .and_then(|t| t.as_str())
        .ok_or_else(|| ForgeError::Validation("missing type field in stream event".into()))?;
    match typ {
        "system" => Ok(Some(serde_json::from_value(value)
            .map_err(|e| ForgeError::Process(format!("JSON parse error: {e}")))?)),
        "assistant" => Ok(Some(serde_json::from_value(value)
            .map_err(|e| ForgeError::Process(format!("JSON parse error: {e}")))?)),
        "user" => Ok(Some(serde_json::from_value(value)
            .map_err(|e| ForgeError::Process(format!("JSON parse error: {e}")))?)),
        "result" => Ok(Some(serde_json::from_value(value)
            .map_err(|e| ForgeError::Process(format!("JSON parse error: {e}")))?)),
        "error" => Ok(Some(serde_json::from_value(value)
            .map_err(|e| ForgeError::Process(format!("JSON parse error: {e}")))?)),
        other => Err(ForgeError::Validation(format!("unknown event type: {other}"))),
    }
}
