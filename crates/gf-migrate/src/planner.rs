//! Migration planner: groups findings by file, prioritizes, and produces a MigrationPlan.

use std::collections::BTreeMap;

use crate::artifact::{self, ScanArtifact};
use crate::prompt::generate_prompt;
use crate::task::{Finding, FindingSource, MigrationTask, Priority, TaskStatus};

/// A complete migration plan: ordered list of tasks ready for execution.
#[derive(Debug)]
pub struct MigrationPlan {
    /// Tasks sorted by priority (blocking first, then high, medium, low).
    pub tasks: Vec<MigrationTask>,
    /// Total number of actionable findings across all tasks.
    pub total_findings: usize,
    /// Number of files that will be modified.
    pub total_files: usize,
    /// Number of info-only tasks that were skipped.
    pub skipped_info_only: usize,
}

impl MigrationPlan {
    /// Summary string for display.
    pub fn summary(&self) -> String {
        format!(
            "{} tasks across {} files ({} findings, {} info-only skipped)",
            self.tasks.len(),
            self.total_files,
            self.total_findings,
            self.skipped_info_only,
        )
    }
}

/// Build a migration plan from a parsed scan artifact.
///
/// Groups all findings (deprecated APIs + Liquid Glass) by file path,
/// assigns priority from the highest-severity finding in each group,
/// generates a Claude Code prompt per task, and sorts tasks by priority.
pub fn plan_migration(artifact: &ScanArtifact) -> MigrationPlan {
    // Collect all findings into a map keyed by normalized file path.
    let mut by_file: BTreeMap<String, Vec<Finding>> = BTreeMap::new();

    // 1. deprecated_apis (rich entries with replacement + notes)
    for api in &artifact.findings.deprecated_apis {
        let path = artifact::normalize_artifact_path(&api.file);
        by_file.entry(path).or_default().push(Finding {
            symbol: api.symbol.clone(),
            line: api.line,
            severity: api.severity.clone(),
            replacement: api.replacement.clone(),
            notes: api.notes.clone(),
            source: FindingSource::DeprecatedApi,
        });
    }

    // 2. liquidGlass.findings (migration and anti-pattern entries)
    if let Some(ref lg) = artifact.liquid_glass {
        for finding in &lg.findings {
            let path = artifact::normalize_artifact_path(&finding.file);
            // Use the pattern as the symbol if present, otherwise the kind.
            let symbol = finding
                .pattern
                .clone()
                .or_else(|| finding.kind.clone())
                .unwrap_or_else(|| "unknown".to_string());
            by_file.entry(path).or_default().push(Finding {
                symbol,
                line: finding.line,
                severity: finding.severity.clone(),
                replacement: finding.replacement.clone(),
                notes: finding.notes.clone(),
                source: FindingSource::LiquidGlass,
            });
        }
    }

    let total_files = by_file.len();
    let mut total_findings: usize = 0;
    let mut skipped_info_only: usize = 0;
    let mut tasks: Vec<MigrationTask> = Vec::with_capacity(by_file.len());

    for (file_path, mut findings) in by_file {
        // Sort findings by line number within the file for stable prompt ordering.
        findings.sort_by_key(|f| f.line);

        // Determine task priority from highest-severity finding.
        let priority = findings
            .iter()
            .map(|f| Priority::from_severity(&f.severity))
            .min()
            .unwrap_or(Priority::Info);

        let prompt = generate_prompt(&file_path, &findings);

        let mut task = MigrationTask {
            file_path,
            findings,
            prompt,
            priority,
            status: TaskStatus::Pending,
            persona_id: None,
        };

        // Skip tasks where every finding is info-only (nothing actionable).
        if task.is_info_only() {
            task.status = TaskStatus::Skipped {
                reason: "all findings are info-only, no changes needed".into(),
            };
            skipped_info_only += 1;
        } else {
            total_findings += task.actionable_count();
        }

        tasks.push(task);
    }

    // Sort by priority (blocking first), then by file path for deterministic ordering.
    tasks.sort_by(|a, b| a.priority.cmp(&b.priority).then(a.file_path.cmp(&b.file_path)));

    MigrationPlan {
        tasks,
        total_findings,
        total_files,
        skipped_info_only,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::artifact::parse_artifact_str;

    const SAMPLE: &str = r#"{
        "findings": {
            "deprecated_apis": [
                {
                    "symbol": "keyWindow",
                    "file": "Sources/App/SceneDelegate.swift",
                    "line": 12,
                    "severity": "high",
                    "replacement": "UIWindowScene.keyWindow",
                    "notes": "Use window scene API."
                },
                {
                    "symbol": "UIScreen.main.bounds",
                    "file": "Sources/App/LayoutHelper.swift",
                    "line": 45,
                    "severity": "medium",
                    "replacement": "Window or scene bounds",
                    "notes": "Prefer scene-relative bounds."
                },
                {
                    "symbol": "statusBarFrame",
                    "file": "Sources/App/SceneDelegate.swift",
                    "line": 30,
                    "severity": "medium",
                    "replacement": "statusBarManager",
                    "notes": "Prefer safe area."
                }
            ]
        },
        "liquidGlass": {
            "readinessLevel": "partial",
            "antiPatternCount": 1,
            "migrationTargetCount": 0,
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
        }
    }"#;

    #[test]
    fn plan_groups_by_file() {
        let artifact = parse_artifact_str(SAMPLE).unwrap();
        let plan = plan_migration(&artifact);
        // 3 unique files: SceneDelegate, LayoutHelper, Theme
        assert_eq!(plan.total_files, 3);
    }

    #[test]
    fn plan_sorts_by_priority() {
        let artifact = parse_artifact_str(SAMPLE).unwrap();
        let plan = plan_migration(&artifact);
        let actionable: Vec<&MigrationTask> = plan
            .tasks
            .iter()
            .filter(|t| !t.is_info_only())
            .collect();
        // High-priority tasks should come before medium.
        assert!(actionable.len() >= 2);
        assert!(actionable[0].priority <= actionable[1].priority);
    }

    #[test]
    fn plan_merges_findings_per_file() {
        let artifact = parse_artifact_str(SAMPLE).unwrap();
        let plan = plan_migration(&artifact);
        // SceneDelegate.swift has 2 deprecated_apis findings.
        let sd = plan
            .tasks
            .iter()
            .find(|t| t.file_path.contains("SceneDelegate"))
            .unwrap();
        assert_eq!(sd.findings.len(), 2);
        // Findings should be sorted by line: 12, 30.
        assert_eq!(sd.findings[0].line, 12);
        assert_eq!(sd.findings[1].line, 30);
    }

    #[test]
    fn plan_theme_has_mixed_findings() {
        let artifact = parse_artifact_str(SAMPLE).unwrap();
        let plan = plan_migration(&artifact);
        let theme = plan
            .tasks
            .iter()
            .find(|t| t.file_path.contains("Theme"))
            .unwrap();
        // Theme.swift has 2 LiquidGlass findings (one actionable, one info).
        assert_eq!(theme.findings.len(), 2);
        assert_eq!(theme.actionable_count(), 1);
        assert!(!theme.is_info_only());
    }

    #[test]
    fn plan_counts_actionable_findings() {
        let artifact = parse_artifact_str(SAMPLE).unwrap();
        let plan = plan_migration(&artifact);
        // 3 deprecated_apis + 1 actionable LiquidGlass = 4 actionable.
        assert_eq!(plan.total_findings, 4);
    }

    #[test]
    fn empty_artifact_produces_empty_plan() {
        let artifact = parse_artifact_str("{}").unwrap();
        let plan = plan_migration(&artifact);
        assert!(plan.tasks.is_empty());
        assert_eq!(plan.total_findings, 0);
    }

    #[test]
    fn plan_tasks_have_no_persona_by_default() {
        let artifact = parse_artifact_str(SAMPLE).unwrap();
        let plan = plan_migration(&artifact);
        for task in &plan.tasks {
            assert!(task.persona_id.is_none());
        }
    }
}
