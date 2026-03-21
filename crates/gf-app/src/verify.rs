//! Verify command: compare before/after scan artifacts.

use std::path::Path;

use anyhow::{Context, Result, bail};
use serde_json::Value;

/// Score tier coloring for terminal display.
fn score_label(score: u64) -> &'static str {
    match score {
        0..=49 => "red",
        50..=74 => "yellow",
        75..=100 => "green",
        _ => "unknown",
    }
}

/// Extract readiness score from an artifact JSON value.
fn extract_score(artifact: &Value, label: &str) -> Result<u64> {
    artifact
        .get("readiness")
        .and_then(|r| r.get("score"))
        .and_then(|s| s.as_u64())
        .with_context(|| format!("could not extract readiness.score from {label} artifact"))
}

/// Extract total finding count from an artifact, if available.
fn extract_finding_count(artifact: &Value) -> Option<u64> {
    // Try common shapes: findings[] array length, or readiness.finding_count
    if let Some(findings) = artifact.get("findings").and_then(|f| f.as_array()) {
        return Some(findings.len() as u64);
    }
    if let Some(count) = artifact
        .get("readiness")
        .and_then(|r| r.get("finding_count"))
        .and_then(|c| c.as_u64())
    {
        return Some(count);
    }
    // Try summary.total_findings
    artifact
        .get("summary")
        .and_then(|s| s.get("total_findings"))
        .and_then(|c| c.as_u64())
}

/// Run the verification comparison between two artifact files.
pub fn run_verify(before_path: &Path, after_path: &Path) -> Result<()> {
    if !before_path.exists() {
        bail!("before artifact does not exist: {}", before_path.display());
    }
    if !after_path.exists() {
        bail!("after artifact does not exist: {}", after_path.display());
    }

    let before_text = std::fs::read_to_string(before_path)
        .with_context(|| format!("failed to read {}", before_path.display()))?;
    let after_text = std::fs::read_to_string(after_path)
        .with_context(|| format!("failed to read {}", after_path.display()))?;

    let before: Value = serde_json::from_str(&before_text)
        .with_context(|| format!("failed to parse JSON from {}", before_path.display()))?;
    let after: Value = serde_json::from_str(&after_text)
        .with_context(|| format!("failed to parse JSON from {}", after_path.display()))?;

    let before_score = extract_score(&before, "before")?;
    let after_score = extract_score(&after, "after")?;
    let delta = after_score as i64 - before_score as i64;

    let delta_str = if delta >= 0 {
        format!("+{delta}")
    } else {
        format!("{delta}")
    };

    println!("=== GlassForge Verification ===");
    println!(
        "Before: {}/100 ({})",
        before_score,
        score_label(before_score)
    );
    println!(
        "After:  {}/100 ({})",
        after_score,
        score_label(after_score)
    );
    println!("Delta:  {} points", delta_str);

    // Compare finding counts if available.
    let before_findings = extract_finding_count(&before);
    let after_findings = extract_finding_count(&after);

    if let (Some(bf), Some(af)) = (before_findings, after_findings) {
        let finding_delta = af as i64 - bf as i64;
        let finding_delta_str = if finding_delta >= 0 {
            format!("+{finding_delta}")
        } else {
            format!("{finding_delta}")
        };
        println!();
        println!("Findings: {} -> {} ({})", bf, af, finding_delta_str);
    }

    Ok(())
}
