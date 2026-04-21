// SS005 positive: downcasts to narrower types.
#![allow(dead_code)]
fn truncate(x: u128) -> u64 {
    x as u64
}
fn to_128(y: u64) -> u128 {
    // Widening cast is fine in practice but still flagged at Low severity
    // since `as` is unsafe for untrusted values.
    y as u128
}

struct W;
impl W {
    fn to_u128(&self) -> u128 { 0 }
}
fn downcast(w: &W) -> u128 {
    w.to_u128()
}
