use anyhow::Result;
use clap::{Parser, ValueEnum};
use soroban_scan::{scan_path, ScanConfig, Severity};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(
    name = "soroban-scan",
    version,
    about = "Static analyzer for Soroban smart contracts",
    long_about = "Detects common security bug classes seen in real Soroban audits \
                  (TTL-expiry, auth bypass, reentrancy, bitmap desync, truncation, \
                  CEI violations)."
)]
struct Cli {
    /// Path to a Rust file or a workspace root.
    path: PathBuf,

    /// Output format.
    #[arg(long, value_enum, default_value_t = Format::Human)]
    format: Format,

    /// Comma-separated list of rule IDs to skip (e.g. `SS006,SS010`).
    #[arg(long)]
    skip: Option<String>,

    /// Include test files in the scan.
    #[arg(long)]
    include_tests: bool,

    /// Exit non-zero if any finding of the given severity (or higher) is reported.
    #[arg(long, value_enum, default_value_t = FailOn::Never)]
    fail_on: FailOn,
}

#[derive(Clone, Debug, ValueEnum)]
enum Format {
    Human,
    Json,
}

#[derive(Clone, Debug, ValueEnum, PartialEq, Eq)]
enum FailOn {
    Never,
    High,
    Medium,
    Low,
    Any,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    let cfg = ScanConfig {
        disabled_rules: cli
            .skip
            .as_deref()
            .map(|s| s.split(',').map(|x| x.trim().to_string()).collect())
            .unwrap_or_default(),
        include_tests: cli.include_tests,
    };

    let scan = scan_path(&cli.path, &cfg)?;

    match cli.format {
        Format::Human => print!("{}", soroban_scan::report::human(&scan)),
        Format::Json => println!("{}", serde_json::to_string_pretty(&scan.findings)?),
    }

    // Exit code.
    let threshold = match cli.fail_on {
        FailOn::Never => return Ok(()),
        FailOn::High => Some(Severity::High),
        FailOn::Medium => Some(Severity::Medium),
        FailOn::Low => Some(Severity::Low),
        FailOn::Any => Some(Severity::Info),
    };
    let Some(t) = threshold else { return Ok(()) };
    let has_hit = scan.findings.iter().any(|f| sev_ord(f.severity) <= sev_ord(t));
    if has_hit {
        std::process::exit(2);
    }
    Ok(())
}

fn sev_ord(s: Severity) -> u8 {
    match s {
        Severity::High => 0,
        Severity::Medium => 1,
        Severity::Low => 2,
        Severity::Info => 3,
    }
}
