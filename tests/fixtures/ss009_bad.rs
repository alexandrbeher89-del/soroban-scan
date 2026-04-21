// SS009 positive: (a / b) * c precision loss.
#![allow(dead_code)]

fn scale(a: u128, b: u128, c: u128) -> u128 {
    (a / b) * c
}

fn scale_ok(a: u128, b: u128, c: u128) -> u128 {
    (a * c) / b
}
