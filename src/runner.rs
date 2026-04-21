use crate::finding::Finding;
use crate::rules::Rule;
use crate::walker;
use anyhow::{Context, Result};
use std::path::{Path, PathBuf};

#[derive(Default)]
pub struct ScanConfig {
    /// Rule ids to disable.
    pub disabled_rules: Vec<String>,
    /// Include test files (default false for audit-style scans).
    pub include_tests: bool,
}

pub struct Scan {
    pub findings: Vec<Finding>,
    pub files_scanned: usize,
    pub errors: Vec<(PathBuf, String)>,
}

fn is_test_path(p: &Path) -> bool {
    p.components()
        .any(|c| matches!(c.as_os_str().to_str(), Some("tests") | Some("fuzz")))
}

pub fn scan_path(root: &Path, cfg: &ScanConfig) -> Result<Scan> {
    let rules = crate::rules::all_rules();
    let enabled: Vec<&dyn Rule> = rules
        .iter()
        .filter(|r| !cfg.disabled_rules.iter().any(|d| d == r.id()))
        .map(|b| b.as_ref())
        .collect();

    let mut files = walker::collect_rust_files(root);
    if !cfg.include_tests {
        files.retain(|p| !is_test_path(p));
    }

    let mut findings = Vec::new();
    let mut errors = Vec::new();
    for file in &files {
        match scan_one(file, &enabled) {
            Ok(mut fs) => findings.append(&mut fs),
            Err(e) => errors.push((file.clone(), format!("{e:#}"))),
        }
    }
    Ok(Scan { findings, files_scanned: files.len(), errors })
}

pub fn scan_file(path: &Path) -> Result<Vec<Finding>> {
    let rules = crate::rules::all_rules();
    let rules_ref: Vec<&dyn Rule> = rules.iter().map(|b| b.as_ref()).collect();
    scan_one(path, &rules_ref)
}

fn scan_one(path: &Path, rules: &[&dyn Rule]) -> Result<Vec<Finding>> {
    let src = std::fs::read_to_string(path)
        .with_context(|| format!("reading {}", path.display()))?;
    let file = syn::parse_file(&src)
        .with_context(|| format!("parsing {}", path.display()))?;
    let mut findings = Vec::new();
    for rule in rules {
        rule.run(path, &file, &src, &mut findings);
    }
    Ok(findings)
}
