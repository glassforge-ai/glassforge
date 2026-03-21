//! Parse GlassForge scan artifact JSON into structured types.
//!
//! The artifact has two arrays of interest:
//! - `findings.deprecated_apis`: deprecated API findings with replacement info
//! - `liquidGlass.findings`: Liquid Glass migration/anti-pattern findings

use serde::Deserialize;
use std::path::Path;

use gf_core::ForgeError;

/// Top-level GlassForge scan artifact.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScanArtifact {
    /// Schema version string (e.g. "2025.02.2").
    #[serde(default)]
    pub schema_version: Option<String>,

    /// Scan generation timestamp.
    #[serde(default, rename = "generatedAt")]
    pub generated_at: Option<String>,

    /// Findings section containing deprecated API hits.
    #[serde(default)]
    pub findings: Findings,

    /// Liquid Glass readiness section with its own findings.
    #[serde(default, rename = "liquidGlass")]
    pub liquid_glass: Option<LiquidGlassSection>,

    /// Metadata about the scan.
    #[serde(default)]
    pub meta: Option<ScanMeta>,
}

/// The findings block within the artifact.
#[derive(Debug, Default, Deserialize)]
pub struct Findings {
    /// Rich deprecated API entries (snake_case key in JSON).
    #[serde(default)]
    pub deprecated_apis: Vec<DeprecatedApiFinding>,

    /// Compact deprecated API entries (camelCase key in JSON).
    #[serde(default, rename = "deprecatedApis")]
    pub deprecated_apis_compact: Vec<DeprecatedApiCompact>,
}

/// A deprecated API finding with replacement guidance.
#[derive(Debug, Clone, Deserialize)]
pub struct DeprecatedApiFinding {
    pub symbol: String,
    pub file: String,
    pub line: u32,
    pub severity: String,
    #[serde(default)]
    pub replacement: Option<String>,
    #[serde(default)]
    pub notes: Option<String>,
}

/// Compact deprecated API entry (from the `deprecatedApis` camelCase array).
/// These lack replacement/notes but have a `kind` field.
#[derive(Debug, Clone, Deserialize)]
pub struct DeprecatedApiCompact {
    pub symbol: String,
    pub file: String,
    pub line: u32,
    pub severity: String,
    #[serde(default)]
    pub kind: Option<String>,
}

/// Liquid Glass readiness section.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LiquidGlassSection {
    #[serde(default)]
    pub readiness_level: Option<String>,
    #[serde(default)]
    pub findings: Vec<LiquidGlassFinding>,
    #[serde(default)]
    pub anti_pattern_count: u32,
    #[serde(default)]
    pub migration_target_count: u32,
    #[serde(default)]
    pub positive_signal_count: u32,
}

/// A single Liquid Glass finding.
#[derive(Debug, Clone, Deserialize)]
pub struct LiquidGlassFinding {
    pub file: String,
    pub line: u32,
    pub severity: String,
    #[serde(default)]
    pub category: Option<String>,
    #[serde(default)]
    pub kind: Option<String>,
    #[serde(default)]
    pub pattern: Option<String>,
    #[serde(default)]
    pub replacement: Option<String>,
    #[serde(default)]
    pub notes: Option<String>,
}

/// Scan metadata.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScanMeta {
    #[serde(default, alias = "repo_name")]
    pub repo_name: Option<String>,
    #[serde(default)]
    pub branch: Option<String>,
    #[serde(default)]
    pub commit: Option<String>,
    #[serde(default, alias = "cli_version")]
    pub cli_version: Option<String>,
    #[serde(default, alias = "target_sdk")]
    pub target_sdk: Option<u32>,
}

/// Parse a GlassForge artifact from a JSON file on disk.
pub fn parse_artifact(path: &Path) -> Result<ScanArtifact, ForgeError> {
    let contents = std::fs::read_to_string(path).map_err(|e| {
        ForgeError::Validation(format!("failed to read artifact at {}: {}", path.display(), e))
    })?;
    parse_artifact_str(&contents)
}

/// Parse a GlassForge artifact from a JSON string.
pub fn parse_artifact_str(json: &str) -> Result<ScanArtifact, ForgeError> {
    serde_json::from_str(json)
        .map_err(|e| ForgeError::Validation(format!("invalid GlassForge artifact JSON: {}", e)))
}

/// Normalize a file path from the artifact (which uses escaped forward slashes).
/// The JSON serialization uses `\/` which serde_json handles, but we also strip
/// leading `./` or `Sources/` prefix if the user's repo root differs.
pub fn normalize_artifact_path(raw: &str) -> String {
    raw.replace("\\/", "/")
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE_ARTIFACT: &str = r#"{
        "schemaVersion": "2025.02.2",
        "generatedAt": "2026-02-28T04:39:24.012Z",
        "findings": {
            "deprecated_apis": [
                {
                    "symbol": "keyWindow",
                    "file": "Sources/App/SceneDelegate.swift",
                    "line": 12,
                    "severity": "high",
                    "replacement": "UIWindowScene.keyWindow",
                    "notes": "keyWindow deprecated; use window scene API."
                },
                {
                    "symbol": "UIScreen.main.bounds",
                    "file": "Sources/App/LayoutHelper.swift",
                    "line": 45,
                    "severity": "medium",
                    "replacement": "Window or scene bounds",
                    "notes": "Prefer scene or view-relative bounds."
                }
            ],
            "deprecatedApis": [
                {
                    "symbol": "keyWindow",
                    "file": "Sources/App/SceneDelegate.swift",
                    "line": 12,
                    "severity": "high",
                    "kind": "deprecated"
                }
            ]
        },
        "liquidGlass": {
            "readinessLevel": "partial",
            "antiPatternCount": 1,
            "migrationTargetCount": 1,
            "positiveSignalCount": 1,
            "findings": [
                {
                    "file": "Sources/App/Theme.swift",
                    "line": 88,
                    "severity": "high",
                    "category": "anti_pattern",
                    "kind": "manualBlur",
                    "pattern": ".blur(radius:",
                    "replacement": ".glassEffect()",
                    "notes": "Replace manual blur with native glassEffect."
                },
                {
                    "file": "Sources/App/Theme.swift",
                    "line": 42,
                    "severity": "info",
                    "category": "positive",
                    "kind": "glassEffect",
                    "pattern": ".glassEffect(",
                    "notes": "Good: native Liquid Glass adoption."
                }
            ]
        },
        "meta": {
            "repo_name": "MyApp",
            "branch": "main",
            "commit": "abc1234",
            "cliVersion": "0.2.0",
            "target_sdk": 26
        }
    }"#;

    #[test]
    fn parse_deprecated_apis() {
        let artifact = parse_artifact_str(SAMPLE_ARTIFACT).unwrap();
        assert_eq!(artifact.findings.deprecated_apis.len(), 2);
        let first = &artifact.findings.deprecated_apis[0];
        assert_eq!(first.symbol, "keyWindow");
        assert_eq!(first.severity, "high");
        assert_eq!(
            first.replacement.as_deref(),
            Some("UIWindowScene.keyWindow")
        );
    }

    #[test]
    fn parse_compact_deprecated_apis() {
        let artifact = parse_artifact_str(SAMPLE_ARTIFACT).unwrap();
        assert_eq!(artifact.findings.deprecated_apis_compact.len(), 1);
        assert_eq!(
            artifact.findings.deprecated_apis_compact[0].kind.as_deref(),
            Some("deprecated")
        );
    }

    #[test]
    fn parse_liquid_glass_findings() {
        let artifact = parse_artifact_str(SAMPLE_ARTIFACT).unwrap();
        let lg = artifact.liquid_glass.unwrap();
        assert_eq!(lg.findings.len(), 2);
        assert_eq!(lg.anti_pattern_count, 1);
        let anti = &lg.findings[0];
        assert_eq!(anti.category.as_deref(), Some("anti_pattern"));
        assert_eq!(anti.replacement.as_deref(), Some(".glassEffect()"));
    }

    #[test]
    fn parse_meta() {
        let artifact = parse_artifact_str(SAMPLE_ARTIFACT).unwrap();
        let meta = artifact.meta.unwrap();
        assert_eq!(meta.repo_name.as_deref(), Some("MyApp"));
        assert_eq!(meta.target_sdk, Some(26));
    }

    #[test]
    fn invalid_json_returns_validation_error() {
        let result = parse_artifact_str("not json");
        assert!(result.is_err());
        match result.unwrap_err() {
            ForgeError::Validation(msg) => {
                assert!(msg.contains("invalid GlassForge artifact JSON"));
            }
            other => panic!("expected Validation, got {:?}", other),
        }
    }

    #[test]
    fn empty_artifact_parses_with_defaults() {
        let artifact = parse_artifact_str("{}").unwrap();
        assert!(artifact.findings.deprecated_apis.is_empty());
        assert!(artifact.liquid_glass.is_none());
    }
}
