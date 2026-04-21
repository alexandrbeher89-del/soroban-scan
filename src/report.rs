use crate::finding::{Finding, Severity};
use crate::runner::Scan;
use colored::Colorize;

pub fn human(scan: &Scan) -> String {
    use std::fmt::Write as _;
    let mut s = String::new();
    let high = scan.findings.iter().filter(|f| f.severity == Severity::High).count();
    let med = scan.findings.iter().filter(|f| f.severity == Severity::Medium).count();
    let low = scan.findings.iter().filter(|f| f.severity == Severity::Low).count();
    let info = scan.findings.iter().filter(|f| f.severity == Severity::Info).count();

    writeln!(
        s,
        "{}",
        format!(
            "soroban-scan: {} files scanned, {} findings ({} high, {} med, {} low, {} info)",
            scan.files_scanned,
            scan.findings.len(),
            high, med, low, info
        )
        .bold()
    )
    .unwrap();

    if !scan.errors.is_empty() {
        writeln!(s, "{}", format!("{} parse error(s):", scan.errors.len()).yellow()).unwrap();
        for (p, e) in &scan.errors {
            writeln!(s, "  {}: {}", p.display(), e).unwrap();
        }
    }

    let mut sorted: Vec<&Finding> = scan.findings.iter().collect();
    sorted.sort_by_key(|f| (sev_rank(f.severity), f.file.clone(), f.line));
    for f in sorted {
        let tag = match f.severity {
            Severity::High => "HIGH".red().bold().to_string(),
            Severity::Medium => "MED ".yellow().bold().to_string(),
            Severity::Low => "LOW ".blue().to_string(),
            Severity::Info => "INFO".dimmed().to_string(),
        };
        writeln!(
            s,
            "{} [{}] {}:{}:{}\n    {}\n    {}",
            tag,
            f.id,
            f.file.display(),
            f.line,
            f.column,
            f.description,
            f.snippet,
        )
        .unwrap();
        if let Some(n) = &f.note {
            for line in n.lines() {
                writeln!(s, "    hint: {line}").unwrap();
            }
        }
        writeln!(s).unwrap();
    }
    s
}

fn sev_rank(s: Severity) -> u8 {
    match s {
        Severity::High => 0,
        Severity::Medium => 1,
        Severity::Low => 2,
        Severity::Info => 3,
    }
}
