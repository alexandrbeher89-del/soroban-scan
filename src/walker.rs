use std::path::{Path, PathBuf};
use walkdir::WalkDir;

/// Recursively collect all `.rs` files under `root`, skipping common non-source
/// directories (`target/`, `.git/`, and anything matching `fuzz/fuzz_targets`
/// by default). Test files are included because audit targets sometimes put
/// reference impls in `tests/`, but the caller can filter afterwards.
pub fn collect_rust_files(root: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    for entry in WalkDir::new(root).follow_links(false).into_iter().filter_map(Result::ok) {
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        if path.extension().and_then(|e| e.to_str()) != Some("rs") {
            continue;
        }
        if path
            .components()
            .any(|c| matches!(c.as_os_str().to_str(), Some("target" | ".git" | "node_modules")))
        {
            continue;
        }
        out.push(path.to_path_buf());
    }
    out
}

/// Convert a `proc_macro2::Span` into (line, column) assuming spans have
/// locations enabled (`proc-macro2` feature `span-locations`).
pub fn span_loc(span: proc_macro2::Span) -> (usize, usize) {
    let start = span.start();
    (start.line, start.column.saturating_add(1))
}

/// Extract a short one-line snippet from a source file at the given 1-based
/// line. Long lines are trimmed to 200 chars. If the file cannot be read or
/// the line is out of bounds, returns an empty string.
pub fn snippet_for(file: &Path, line: usize) -> String {
    let Ok(src) = std::fs::read_to_string(file) else {
        return String::new();
    };
    let Some(l) = src.lines().nth(line.saturating_sub(1)) else {
        return String::new();
    };
    let trimmed = l.trim();
    if trimmed.len() > 200 {
        format!("{}...", &trimmed[..200])
    } else {
        trimmed.to_string()
    }
}
