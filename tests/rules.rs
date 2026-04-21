//! Rule regression tests — each fixture should produce at least one finding
//! with the expected rule id.

use std::path::Path;

fn scan_fixture(name: &str) -> Vec<soroban_scan::Finding> {
    let p = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures").join(name);
    soroban_scan::scan_file(&p).expect("parse fixture")
}

fn has_id(findings: &[soroban_scan::Finding], id: &str) -> bool {
    findings.iter().any(|f| f.id == id)
}

#[test]
fn ss001_fires_on_unwrap_or_default() {
    let f = scan_fixture("ss001_bad.rs");
    assert!(has_id(&f, "SS001"), "expected SS001 in fixture, got {f:#?}");
}

#[test]
fn ss002_fires_on_missing_require_auth() {
    let f = scan_fixture("ss002_bad.rs");
    assert!(has_id(&f, "SS002"), "expected SS002 in fixture, got {f:#?}");
    // `transfer_ok` calls require_auth, `get_balance` is a view — neither
    // should produce SS002 on its own, but `transfer` must.
    let offenders: Vec<_> = f.iter().filter(|x| x.id == "SS002").collect();
    assert!(offenders.iter().any(|x| x.snippet.contains("transfer")));
}

#[test]
fn ss005_fires_on_as_cast_narrow() {
    let f = scan_fixture("ss005_bad.rs");
    assert!(has_id(&f, "SS005"), "expected SS005 in fixture, got {f:#?}");
}

#[test]
fn ss007_fires_on_duplicate_discriminant() {
    let f = scan_fixture("ss007_bad.rs");
    assert!(has_id(&f, "SS007"), "expected SS007 in fixture, got {f:#?}");
}

#[test]
fn ss009_fires_on_div_before_mul() {
    let f = scan_fixture("ss009_bad.rs");
    assert!(has_id(&f, "SS009"), "expected SS009 in fixture, got {f:#?}");
}

#[test]
fn ss012_fires_on_unwrap_in_contractimpl() {
    let f = scan_fixture("ss012_bad.rs");
    assert!(has_id(&f, "SS012"), "expected SS012 in fixture, got {f:#?}");
}
