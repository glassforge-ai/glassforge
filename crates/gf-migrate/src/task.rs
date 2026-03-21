//! MigrationTask: unit of work for one file's worth of findings.

use serde::{Deserialize, Serialize};

/// A single actionable finding to fix in a file.
/// Unified from both deprecated API and Liquid Glass sources.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Finding {
    /// The deprecated symbol or anti-pattern matched.
    pub symbol: String,
    /// Line number in the source file.
    pub line: u32,
    /// Severity: "blocking", "high", "medium", "low", "info".
    pub severity: String,
    /// Suggested replacement code/API.
    pub replacement: Option<String>,
    /// Human-readable notes about the migration.
    pub notes: Option<String>,
    /// Source of the finding: "deprecated_api" or "liquid_glass".
    pub source: FindingSource,
}

/// Where the finding originated in the scan artifact.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FindingSource {
    DeprecatedApi,
    LiquidGlass,
}

/// Priority level derived from severity. Lower numeric value = higher priority.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Priority {
    Blocking = 0,
    High = 1,
    Medium = 2,
    Low = 3,
    Info = 4,
}

impl Priority {
    /// Parse a severity string into a priority level.
    pub fn from_severity(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "blocking" | "critical" => Priority::Blocking,
            "high" => Priority::High,
            "medium" => Priority::Medium,
            "low" => Priority::Low,
            _ => Priority::Info,
        }
    }
}

impl std::fmt::Display for Priority {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Priority::Blocking => write!(f, "blocking"),
            Priority::High => write!(f, "high"),
            Priority::Medium => write!(f, "medium"),
            Priority::Low => write!(f, "low"),
            Priority::Info => write!(f, "info"),
        }
    }
}

/// Execution status of a migration task.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TaskStatus {
    /// Not yet started.
    Pending,
    /// Agent is currently working on it.
    Running,
    /// Agent completed successfully.
    Completed {
        /// Summary output from the agent.
        summary: String,
    },
    /// Agent failed or was killed.
    Failed {
        /// Error description.
        error: String,
    },
    /// Skipped (e.g. info-only findings, no actionable changes).
    Skipped {
        reason: String,
    },
}

impl std::fmt::Display for TaskStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TaskStatus::Pending => write!(f, "pending"),
            TaskStatus::Running => write!(f, "running"),
            TaskStatus::Completed { .. } => write!(f, "completed"),
            TaskStatus::Failed { error } => write!(f, "failed: {}", error),
            TaskStatus::Skipped { reason } => write!(f, "skipped: {}", reason),
        }
    }
}

/// A migration task: one file with all its findings, a generated prompt, and a priority.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MigrationTask {
    /// Relative path to the source file within the repo.
    pub file_path: String,
    /// All findings for this file, sorted by line number.
    pub findings: Vec<Finding>,
    /// Generated Claude Code prompt for this task.
    pub prompt: String,
    /// Highest priority among the findings in this task.
    pub priority: Priority,
    /// Current execution status.
    pub status: TaskStatus,
    /// Optional persona ID for agent role assignment.
    pub persona_id: Option<String>,
}

impl MigrationTask {
    /// Number of actionable findings (excludes info-only).
    pub fn actionable_count(&self) -> usize {
        self.findings
            .iter()
            .filter(|f| Priority::from_severity(&f.severity) != Priority::Info)
            .count()
    }

    /// True if all findings are info-only (nothing to fix).
    pub fn is_info_only(&self) -> bool {
        self.actionable_count() == 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn priority_ordering() {
        assert!(Priority::Blocking < Priority::High);
        assert!(Priority::High < Priority::Medium);
        assert!(Priority::Medium < Priority::Low);
        assert!(Priority::Low < Priority::Info);
    }

    #[test]
    fn priority_from_severity() {
        assert_eq!(Priority::from_severity("blocking"), Priority::Blocking);
        assert_eq!(Priority::from_severity("critical"), Priority::Blocking);
        assert_eq!(Priority::from_severity("HIGH"), Priority::High);
        assert_eq!(Priority::from_severity("medium"), Priority::Medium);
        assert_eq!(Priority::from_severity("low"), Priority::Low);
        assert_eq!(Priority::from_severity("info"), Priority::Info);
        assert_eq!(Priority::from_severity("unknown"), Priority::Info);
    }

    #[test]
    fn actionable_count_excludes_info() {
        let task = MigrationTask {
            file_path: "test.swift".into(),
            findings: vec![
                Finding {
                    symbol: "keyWindow".into(),
                    line: 10,
                    severity: "high".into(),
                    replacement: Some("UIWindowScene.keyWindow".into()),
                    notes: None,
                    source: FindingSource::DeprecatedApi,
                },
                Finding {
                    symbol: ".glassEffect(".into(),
                    line: 20,
                    severity: "info".into(),
                    replacement: None,
                    notes: Some("Good: already adopted.".into()),
                    source: FindingSource::LiquidGlass,
                },
            ],
            prompt: String::new(),
            priority: Priority::High,
            status: TaskStatus::Pending,
            persona_id: None,
        };
        assert_eq!(task.actionable_count(), 1);
        assert!(!task.is_info_only());
    }

    #[test]
    fn persona_id_defaults_to_none() {
        let task = MigrationTask {
            file_path: "test.swift".into(),
            findings: vec![],
            prompt: String::new(),
            priority: Priority::Info,
            status: TaskStatus::Pending,
            persona_id: None,
        };
        assert!(task.persona_id.is_none());
    }

    #[test]
    fn persona_id_can_be_set() {
        let task = MigrationTask {
            file_path: "test.swift".into(),
            findings: vec![],
            prompt: String::new(),
            priority: Priority::Info,
            status: TaskStatus::Pending,
            persona_id: Some("ios-modernizer".into()),
        };
        assert_eq!(task.persona_id.as_deref(), Some("ios-modernizer"));
    }
}
