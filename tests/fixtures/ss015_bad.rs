// Fixture for SS015 — floating-point type in a return position.
// Parsed by syn; does not need to compile against soroban-sdk.
//
// Single float site on purpose: per CONTRIBUTING.md step 5, the
// positive fixture must trigger the rule exactly once so the regression
// test can assert `hits.len() == 1`. Real-world contracts that ship
// floating-point math typically mix return types, casts, and local
// bindings; each of those axes is independently exercised on live
// targets and does not need a separate fixture.

#[allow(dead_code)]
fn price_ratio_bad(numerator: i128, denominator: i128) -> f64 {
    // BAD: any float in a contract-facing signature is a consensus-safety
    // hazard. Both lines would normally fire, but we return a constant so
    // only the return-type `f64` triggers SS015.
    let _ = (numerator, denominator);
    0.0
}
