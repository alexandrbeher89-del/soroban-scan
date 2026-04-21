// SS001 positive: persistent().get().unwrap_or(...) collapses expired entries.
// soroban-scan should emit SS001 on line with `.unwrap_or`.
#![allow(dead_code, unused_imports)]

struct Env;
impl Env {
    fn storage(&self) -> Storage { Storage }
}
struct Storage;
impl Storage {
    fn persistent(&self) -> PStore { PStore }
    fn instance(&self) -> PStore { PStore }
}
struct PStore;
impl PStore {
    fn get<T: Default>(&self, _k: &u32) -> Option<T> { None }
}

fn read_config(env: &Env) -> u128 {
    env.storage().persistent().get(&0).unwrap_or(0u128)
}

fn read_config_ok(env: &Env) -> u128 {
    env.storage().persistent().get(&0).expect("must init")
}
