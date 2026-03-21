//! Prompt generation for Claude Code migration agents.

use crate::task::{Finding, FindingSource, Priority};

/// Generate a Claude Code prompt for migrating a single file.
///
/// The prompt instructs the agent to fix all findings in the file,
/// preserving behavior and function signatures, and verifying compilation.
pub fn generate_prompt(file_path: &str, findings: &[Finding]) -> String {
    let mut sections = Vec::new();

    // Header
    sections.push(format!(
        "You are modernizing an iOS app for iOS 26. \
         Fix the following deprecated APIs and anti-patterns in `{}`:",
        file_path
    ));
    sections.push(String::new());

    // Group findings by source for clearer organization.
    let deprecated: Vec<&Finding> = findings
        .iter()
        .filter(|f| f.source == FindingSource::DeprecatedApi)
        .filter(|f| Priority::from_severity(&f.severity) != Priority::Info)
        .collect();

    let liquid_glass: Vec<&Finding> = findings
        .iter()
        .filter(|f| f.source == FindingSource::LiquidGlass)
        .filter(|f| Priority::from_severity(&f.severity) != Priority::Info)
        .collect();

    if !deprecated.is_empty() {
        sections.push("## Deprecated API Fixes".to_string());
        for f in &deprecated {
            let mut line = format!("- Line {}: Replace `{}`", f.line, f.symbol);
            if let Some(ref replacement) = f.replacement {
                line.push_str(&format!(" with `{}`", replacement));
            }
            if let Some(ref notes) = f.notes {
                line.push_str(&format!(". {}", notes));
            }
            sections.push(line);
        }
        sections.push(String::new());
    }

    if !liquid_glass.is_empty() {
        sections.push("## Liquid Glass Fixes".to_string());
        for f in &liquid_glass {
            let mut line = format!("- Line {}: Replace `{}`", f.line, f.symbol);
            if let Some(ref replacement) = f.replacement {
                line.push_str(&format!(" with `{}`", replacement));
            }
            if let Some(ref notes) = f.notes {
                line.push_str(&format!(". {}", notes));
            }
            sections.push(line);
        }
        sections.push(String::new());
    }

    // Rules
    sections.push("## Rules".to_string());
    sections.push("- Do not change behavior".to_string());
    sections.push("- Keep the same function signatures".to_string());
    sections.push(
        "- Run `swift build` after changes to verify compilation".to_string(),
    );
    sections.push(format!("- Only modify the specified file: `{}`", file_path));
    sections.push(
        "- If a replacement requires new imports, add them at the top of the file".to_string(),
    );
    sections.push(
        "- Prefer scene-based APIs (UIWindowScene) over deprecated app-level APIs".to_string(),
    );

    sections.join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::task::FindingSource;

    #[test]
    fn prompt_contains_file_path() {
        let findings = vec![Finding {
            symbol: "keyWindow".into(),
            line: 12,
            severity: "high".into(),
            replacement: Some("UIWindowScene.keyWindow".into()),
            notes: Some("Use window scene API.".into()),
            source: FindingSource::DeprecatedApi,
        }];
        let prompt = generate_prompt("Sources/App/SceneDelegate.swift", &findings);
        assert!(prompt.contains("Sources/App/SceneDelegate.swift"));
    }

    #[test]
    fn prompt_includes_replacement_and_notes() {
        let findings = vec![Finding {
            symbol: "UIScreen.main.bounds".into(),
            line: 45,
            severity: "medium".into(),
            replacement: Some("Window or scene bounds".into()),
            notes: Some("Prefer scene-relative bounds.".into()),
            source: FindingSource::DeprecatedApi,
        }];
        let prompt = generate_prompt("LayoutHelper.swift", &findings);
        assert!(prompt.contains("Line 45"));
        assert!(prompt.contains("`UIScreen.main.bounds`"));
        assert!(prompt.contains("`Window or scene bounds`"));
        assert!(prompt.contains("Prefer scene-relative bounds."));
    }

    #[test]
    fn prompt_includes_liquid_glass_section() {
        let findings = vec![Finding {
            symbol: ".blur(radius:".into(),
            line: 88,
            severity: "high".into(),
            replacement: Some(".glassEffect()".into()),
            notes: Some("Replace manual blur.".into()),
            source: FindingSource::LiquidGlass,
        }];
        let prompt = generate_prompt("Theme.swift", &findings);
        assert!(prompt.contains("## Liquid Glass Fixes"));
        assert!(prompt.contains("`.glassEffect()`"));
    }

    #[test]
    fn prompt_skips_info_findings() {
        let findings = vec![Finding {
            symbol: ".glassEffect(".into(),
            line: 42,
            severity: "info".into(),
            replacement: None,
            notes: Some("Good: already adopted.".into()),
            source: FindingSource::LiquidGlass,
        }];
        let prompt = generate_prompt("Theme.swift", &findings);
        // Info-only findings should not appear in fix instructions.
        assert!(!prompt.contains("Line 42"));
    }

    #[test]
    fn prompt_includes_rules() {
        let findings = vec![Finding {
            symbol: "keyWindow".into(),
            line: 1,
            severity: "high".into(),
            replacement: None,
            notes: None,
            source: FindingSource::DeprecatedApi,
        }];
        let prompt = generate_prompt("File.swift", &findings);
        assert!(prompt.contains("Do not change behavior"));
        assert!(prompt.contains("swift build"));
        assert!(prompt.contains("Only modify the specified file"));
    }

    #[test]
    fn prompt_handles_missing_replacement() {
        let findings = vec![Finding {
            symbol: "keyWindow".into(),
            line: 5,
            severity: "high".into(),
            replacement: None,
            notes: None,
            source: FindingSource::DeprecatedApi,
        }];
        let prompt = generate_prompt("File.swift", &findings);
        assert!(prompt.contains("Line 5: Replace `keyWindow`"));
        // Should NOT contain "with `" since no replacement.
        assert!(!prompt.contains("with `"));
    }
}
