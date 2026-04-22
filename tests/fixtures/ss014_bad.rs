// Fixture for SS014 — deprecated Soroban TTL API `.bump(...)`.
// Parsed by syn; does not need to compile against soroban-sdk.

#[allow(dead_code)]
fn bump_persistent(env: &Env, key: &DataKey) {
    // BAD: deprecated `bump` — rename to `extend_ttl`.
    env.storage().persistent().bump(key, 100, 1000);
}
