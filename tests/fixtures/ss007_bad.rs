// SS007 positive: duplicate discriminants in #[contracttype] enum.
#![allow(dead_code)]

#[contracttype]
enum DataKey {
    Admin = 0,
    Owner = 0,
    Reserve(u32) = 1,
}

mod contracttype { pub use super::*; }
