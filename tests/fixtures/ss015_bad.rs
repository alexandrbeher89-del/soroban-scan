// Fixture for SS015 — floating-point types in contract code.
// Parsed by syn; does not need to compile against soroban-sdk.

#[allow(dead_code)]
fn price_ratio_bad(numerator: i128, denominator: i128) -> f64 {
    // BAD: f64 arithmetic is not consensus-safe across Wasm runtimes.
    numerator as f64 / denominator as f64
}
