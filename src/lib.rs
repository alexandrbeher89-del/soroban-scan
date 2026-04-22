//! soroban-scan — static analyzer for Soroban smart contracts.
//!
//! The library exposes a small API that lets you run all registered rules over
//! a single file or a whole crate/workspace and collect [`Finding`]s. The
//! binary crate (`soroban-scan`) is a thin wrapper that adds CLI + reporting.

pub mod finding;
pub mod report;
pub mod rules;
pub mod runner;
pub mod sarif;
pub mod walker;

pub use finding::{Finding, Severity};
pub use runner::{scan_file, scan_path, Scan, ScanConfig};
