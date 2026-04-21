// SS002 positive: function with Address param but no require_auth.
#![allow(dead_code)]
struct Address;
impl Address { fn require_auth(&self) {} }
struct Env;

struct C;
#[contractimpl]
impl C {
    pub fn transfer(env: Env, from: Address, to: Address, amount: u128) {
        let _ = (env, from, to, amount);
    }

    pub fn transfer_ok(env: Env, from: Address, to: Address, amount: u128) {
        from.require_auth();
        let _ = (env, to, amount);
    }

    pub fn get_balance(env: Env, user: Address) -> u128 {
        let _ = (env, user);
        0
    }
}

#[allow(non_snake_case)]
mod contractimpl { pub use super::*; }
