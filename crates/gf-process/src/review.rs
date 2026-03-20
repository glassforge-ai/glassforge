//! Code review engine with specialist evaluators.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReviewAspect {
    PrComments,
    TestCoverage,
    ErrorHandling,
    TypeDesign,
    CodeQuality,
    Simplification,
}

impl ReviewAspect {
    pub fn all() -> Vec<ReviewAspect> {
        vec![
            Self::PrComments,
            Self::TestCoverage,
            Self::ErrorHandling,
            Self::TypeDesign,
            Self::CodeQuality,
            Self::Simplification,
        ]
    }

    pub fn display_name(&self) -> &str {
        match self {
            Self::PrComments => "PR Comments Quality",
            Self::TestCoverage => "Test Coverage Adequacy",
            Self::ErrorHandling => "Error Handling Completeness",
            Self::TypeDesign => "Type Design Correctness",
            Self::CodeQuality => "Code Quality (SOLID, DRY)",
            Self::Simplification => "Simplification Opportunities",
        }
    }

    pub fn system_prompt(&self) -> String {
        match self {
            Self::PrComments => "You are a PR review specialist. Evaluate the quality of PR comments, commit messages, and documentation changes. Score confidence 0-100 on how well the PR communicates intent.".into(),
            Self::TestCoverage => "You are a test coverage specialist. Evaluate whether the code has adequate test coverage for new/changed functionality. Check for edge cases, error paths, and integration tests. Score confidence 0-100.".into(),
            Self::ErrorHandling => "You are an error handling specialist. Evaluate whether errors are properly handled, propagated, and communicated. Check for swallowed errors, missing error types, and panic risks. Score confidence 0-100.".into(),
            Self::TypeDesign => "You are a type system specialist. Evaluate type correctness, proper use of generics, newtypes, and whether the type design prevents invalid states. Score confidence 0-100.".into(),
            Self::CodeQuality => "You are a code quality specialist. Evaluate adherence to SOLID principles, DRY, naming conventions, and overall code structure. Score confidence 0-100.".into(),
            Self::Simplification => "You are a simplification specialist. Identify unnecessary complexity, over-engineering, and opportunities for reducing code while maintaining functionality. Score confidence 0-100.".into(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewFinding {
    pub aspect: ReviewAspect,
    pub confidence: u8,
    pub severity: ReviewSeverity,
    pub message: String,
    pub suggestion: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReviewSeverity {
    Critical,
    Important,
    Minor,
}

impl ReviewFinding {
    pub fn severity_from_confidence(confidence: u8) -> ReviewSeverity {
        match confidence {
            90..=100 => ReviewSeverity::Critical,
            80..=89 => ReviewSeverity::Important,
            _ => ReviewSeverity::Minor,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewReport {
    pub findings: Vec<ReviewFinding>,
    pub aspects_reviewed: Vec<ReviewAspect>,
    pub overall_confidence: f64,
}

impl ReviewReport {
    pub fn from_findings(findings: Vec<ReviewFinding>, confidence_threshold: u8) -> Self {
        let aspects_reviewed: Vec<ReviewAspect> = ReviewAspect::all();

        let filtered: Vec<ReviewFinding> = findings
            .into_iter()
            .filter(|f| f.confidence >= confidence_threshold)
            .collect();

        let overall_confidence = if filtered.is_empty() {
            100.0
        } else {
            filtered.iter().map(|f| f.confidence as f64).sum::<f64>() / filtered.len() as f64
        };

        ReviewReport {
            findings: filtered,
            aspects_reviewed,
            overall_confidence,
        }
    }

    pub fn count_by_severity(&self, severity: &ReviewSeverity) -> usize {
        self.findings.iter().filter(|f| &f.severity == severity).count()
    }
}

pub const DEFAULT_CONFIDENCE_THRESHOLD: u8 = 80;
