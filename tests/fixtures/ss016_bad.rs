// Fixture for SS016 — contract `initialize` writes privileged state but
// never calls require_auth on any address, and is not a `__constructor`.
// Parsed by syn; does not need to compile against soroban-sdk.

#[contractimpl]
impl Factory {
    // BAD: privileged two-arg initialiser with no require_auth anywhere.
    // Front-runnable between `deploy` and `invoke initialize`.
    pub fn initialize(e: Env, setter: Address, pair_wasm_hash: BytesN<32>) {
        if has_setter(&e) {
            panic!("already initialized");
        }
        put_setter(&e, &setter);
        put_pair_wasm_hash(&e, pair_wasm_hash);
    }
}
