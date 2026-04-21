use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    /// Potentially loss of funds or protocol bricking. Very high priority.
    High,
    /// Meaningful impact under specific conditions. Triage priority.
    Medium,
    /// Hardening / defense-in-depth. Review but low urgency.
    Low,
    /// Code smell / gas-budget / stylistic. Informational only.
    Info,
}

impl Severity {
    pub fn label(&self) -> &'static str {
        match self {
            Severity::High => "HIGH",
            Severity::Medium => "MED",
            Severity::Low => "LOW",
            Severity::Info => "INFO",
        }
    }
}

/// A single detected issue.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Finding {
    /// Stable, short identifier (e.g. `SS001`).
    pub id: &'static str,
    /// Short human-readable rule name.
    pub name: &'static str,
    /// One-line description of the bug class.
    pub description: &'static str,
    pub severity: Severity,
    /// Absolute or repo-relative file path.
    pub file: PathBuf,
    /// 1-based line number of the offending span.
    pub line: usize,
    /// 1-based column (best-effort).
    pub column: usize,
    /// The offending source snippet (trimmed).
    pub snippet: String,
    /// Rule-specific extra detail, if any.
    pub note: Option<String>,
}
