// Negative fixture for SS015 — none of these must fire.
// Parsed by syn; does not need to compile.

#[allow(dead_code)]
fn price_ratio_good(numerator: i128, denominator: i128, scale: i128) -> i128 {
    // OK: fixed-point integer math. Deterministic on every Wasm runtime.
    numerator.saturating_mul(scale) / denominator
}

#[allow(dead_code)]
fn u256_mul_good(a: u128, b: u128) -> u128 {
    // OK: widening to u256 would be ideal; here we stick to u128 for the
    // fixture. No floats involved.
    a.saturating_mul(b)
}

#[allow(dead_code)]
struct Position {
    // OK: integer scaled by 1e7 (Stellar's native precision).
    amount_scaled_1e7: i128,
}
