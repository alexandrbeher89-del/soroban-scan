//! SARIF 2.1.0 output for GitHub Code Scanning (and compatible tooling).
//!
//! The emitted JSON conforms to the OASIS SARIF 2.1.0 schema. Only the
//! fields GitHub's ingestion code actually reads are populated; everything
//! else is deliberately omitted to keep output small and easy to diff.

use crate::finding::Severity;
use crate::rules::all_rules;
use crate::runner::Scan;
use serde_json::{json, Value};

const SARIF_SCHEMA: &str =
    "https://raw.githubusercontent.com/oasis-tcs/sarif-spec/master/Schemata/sarif-schema-2.1.0.json";
const TOOL_URI: &str = "https://github.com/alexandrbeher89-del/soroban-scan";

pub fn render(scan: &Scan) -> Value {
    let rules: Vec<Value> = all_rules()
        .iter()
        .map(|r| {
            json!({
                "id": r.id(),
                "name": r.name(),
                "shortDescription": { "text": r.description() },
                "fullDescription":  { "text": r.description() },
                "helpUri": format!("{TOOL_URI}#rules"),
                "defaultConfiguration": {
                    // soroban-scan has per-finding severities; this is the
                    // default only used if a result omits a level.
                    "level": "warning"
                }
            })
        })
        .collect();

    // SARIF URIs should be relative to the repo root. Strip the current
    // working directory if it is a prefix of the finding path.
    let cwd = std::env::current_dir().ok();
    let results: Vec<Value> = scan
        .findings
        .iter()
        .map(|f| {
            let rel = cwd
                .as_ref()
                .and_then(|c| f.file.strip_prefix(c).ok())
                .unwrap_or(&f.file);
            let uri = rel.to_string_lossy().replace('\\', "/");
            json!({
                "ruleId": f.id,
                "level": sarif_level(f.severity),
                "message": {
                    "text": format!("{}: {}", f.description, f.snippet)
                },
                "locations": [{
                    "physicalLocation": {
                        "artifactLocation": { "uri": uri },
                        "region": {
                            "startLine": f.line,
                            "startColumn": f.column,
                            "snippet": { "text": f.snippet }
                        }
                    }
                }],
                "properties": f.note.as_ref().map(|n| json!({ "hint": n })).unwrap_or(json!({}))
            })
        })
        .collect();

    json!({
        "$schema": SARIF_SCHEMA,
        "version": "2.1.0",
        "runs": [{
            "tool": {
                "driver": {
                    "name": "soroban-scan",
                    "version": env!("CARGO_PKG_VERSION"),
                    "informationUri": TOOL_URI,
                    "rules": rules
                }
            },
            "results": results
        }]
    })
}

/// Map soroban-scan severity to SARIF "level".
/// GitHub Code Scanning recognises: `error`, `warning`, `note`, `none`.
fn sarif_level(sev: Severity) -> &'static str {
    match sev {
        Severity::High => "error",
        Severity::Medium => "warning",
        Severity::Low => "note",
        Severity::Info => "none",
    }
}
