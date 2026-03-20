use regex::Regex;
use std::sync::LazyLock;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Severity {
    Critical,
    High,
    Medium,
    Low,
}

impl Severity {
    fn rank(&self) -> u8 {
        match self {
            Severity::Critical => 0,
            Severity::High => 1,
            Severity::Medium => 2,
            Severity::Low => 3,
        }
    }
}

#[derive(Debug, Clone)]
pub struct SecurityFinding {
    pub pattern: String,
    pub severity: Severity,
    pub line: usize,
    pub snippet: String,
    pub description: String,
}

struct ScanPattern {
    name: &'static str,
    severity: Severity,
    regex: Regex,
    description: &'static str,
}

static SCAN_PATTERNS: LazyLock<Vec<ScanPattern>> = LazyLock::new(|| {
    vec![
        ScanPattern {
            name: "command_injection",
            severity: Severity::Critical,
            regex: Regex::new(r#"os\.system\(|subprocess\.(call|run|Popen)\(.*f"|exec\(.*shell"#)
                .expect("command_injection regex is valid"),
            description: "Potential command injection via os.system/subprocess/exec",
        },
        ScanPattern {
            name: "xss_dangerous_html",
            severity: Severity::High,
            regex: Regex::new(r#"\.innerHTML\s*=|dangerouslySetInnerHTML|v-html"#)
                .expect("xss_dangerous_html regex is valid"),
            description: "Dangerous HTML assignment may allow XSS",
        },
        ScanPattern {
            name: "eval_injection",
            severity: Severity::Critical,
            regex: Regex::new(r#"eval\(|exec\(|new Function\("#)
                .expect("eval_injection regex is valid"),
            description: "Dynamic code execution via eval/exec/new Function",
        },
        ScanPattern {
            name: "sql_injection",
            severity: Severity::Critical,
            regex: Regex::new(r#"f".*(?i:SELECT|INSERT|UPDATE|DELETE)|"(?i:SELECT).*"\s*\+|\.format\(.*(?i:SELECT)"#)
                .expect("sql_injection regex is valid"),
            description: "Potential SQL injection via string interpolation",
        },
        ScanPattern {
            name: "path_traversal",
            severity: Severity::High,
            regex: Regex::new(r#"\.\./.*open|open\(.*\.\./|os\.path\.join\(.*request"#)
                .expect("path_traversal regex is valid"),
            description: "Path traversal via ../ in file operations",
        },
        ScanPattern {
            name: "pickle_deserialization",
            severity: Severity::High,
            regex: Regex::new(r#"pickle\.loads?\(|yaml\.load\("#)
                .expect("pickle_deserialization regex is valid"),
            description: "Unsafe deserialization via pickle/yaml.load",
        },
        ScanPattern {
            name: "hardcoded_secrets",
            severity: Severity::Medium,
            regex: Regex::new(r#"(?i)(api_key|password|secret|token)\s*=\s*["'][^"'\s$\{][^"']*["']"#)
                .expect("hardcoded_secrets regex is valid"),
            description: "Hardcoded secret or credential in source code",
        },
        ScanPattern {
            name: "insecure_random",
            severity: Severity::Low,
            regex: Regex::new(r#"Math\.random\(\)|random\.random\(\)"#)
                .expect("insecure_random regex is valid"),
            description: "Insecure random number generator used (not suitable for crypto/auth)",
        },
        ScanPattern {
            name: "open_redirect",
            severity: Severity::Medium,
            regex: Regex::new(r#"redirect\(request\.(GET|POST|query)"#)
                .expect("open_redirect regex is valid"),
            description: "Open redirect using unvalidated user input",
        },
    ]
});

pub struct SecurityScanner;

impl SecurityScanner {
    pub fn new() -> Self {
        LazyLock::force(&SCAN_PATTERNS);
        Self
    }

    pub fn scan(&self, code: &str) -> Vec<SecurityFinding> {
        let mut findings = Vec::new();

        for (line_idx, line) in code.lines().enumerate() {
            for pat in SCAN_PATTERNS.iter() {
                if pat.regex.is_match(line) {
                    findings.push(SecurityFinding {
                        pattern: pat.name.to_string(),
                        severity: pat.severity.clone(),
                        line: line_idx + 1,
                        snippet: line.trim().to_string(),
                        description: pat.description.to_string(),
                    });
                }
            }
        }

        findings.sort_by_key(|f| f.severity.rank());
        findings
    }
}

impl Default for SecurityScanner {
    fn default() -> Self {
        Self::new()
    }
}
