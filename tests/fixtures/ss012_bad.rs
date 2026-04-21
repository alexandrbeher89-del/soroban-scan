// SS012 positive: unwrap/panic in #[contractimpl] method.
#![allow(dead_code)]

struct C;
#[contractimpl]
impl C {
    pub fn oops(xs: Option<u32>) -> u32 {
        xs.unwrap()
    }
    pub fn explode() {
        panic!("nope");
    }
    pub fn okay(xs: Option<u32>) -> Result<u32, u32> {
        xs.ok_or(0)
    }
}
mod contractimpl { pub use super::*; }
