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

#[test]
fn ss013_fires_on_ledger_as_randomness() {
    let f = scan_fixture("ss013_bad.rs");
    let hits: Vec<_> = f.iter().filter(|x| x.id == "SS013").collect();
    assert_eq!(hits.len(), 1, "expected exactly one SS013 hit, got {hits:#?}");
}

#[test]
fn ss013_does_not_fire_on_benign_ledger_use() {
    let f = scan_fixture("ss013_good.rs");
    assert!(
        !has_id(&f, "SS013"),
        "SS013 must not fire on comparison / subtraction / non-entropy calls, got {f:#?}"
    );
}

#[test]
fn ss014_fires_on_deprecated_bump() {
    let f = scan_fixture("ss014_bad.rs");
    let hits: Vec<_> = f.iter().filter(|x| x.id == "SS014").collect();
    assert_eq!(hits.len(), 1, "expected exactly one SS014 hit, got {hits:#?}");
}

#[test]
fn ss014_does_not_fire_on_extend_ttl_or_unrelated_bump() {
    let f = scan_fixture("ss014_good.rs");
    assert!(
        !has_id(&f, "SS014"),
        "SS014 must not fire on extend_ttl or non-storage bump, got {f:#?}"
    );
}

#[test]
fn sarif_output_has_valid_shape() {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures");
    let cfg = soroban_scan::ScanConfig {
        include_tests: true,
        ..Default::default()
    };
    let scan = soroban_scan::scan_path(&dir, &cfg).expect("scan fixtures");
    let sarif = soroban_scan::sarif::render(&scan);

    assert_eq!(sarif["version"], "2.1.0");
    assert!(sarif["$schema"].as_str().unwrap().starts_with("https://"));

    let run = &sarif["runs"][0];
    assert_eq!(run["tool"]["driver"]["name"], "soroban-scan");
    let rules = run["tool"]["driver"]["rules"].as_array().expect("rules array");
    assert_eq!(rules.len(), 14, "all 14 rules must appear in SARIF metadata");

    let results = run["results"].as_array().expect("results array");
    assert!(!results.is_empty(), "fixtures should produce findings");
    for r in results {
        let level = r["level"].as_str().unwrap();
        assert!(
            matches!(level, "error" | "warning" | "note" | "none"),
            "invalid SARIF level: {level}"
        );
        assert!(r["ruleId"].is_string());
        let loc = &r["locations"][0]["physicalLocation"];
        assert!(loc["artifactLocation"]["uri"].is_string());
        assert!(loc["region"]["startLine"].is_number());
    }
}
