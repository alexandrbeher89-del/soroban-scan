// Negative fixture for SS014 — none of these must fire.
// Parsed by syn; does not need to compile.

#[allow(dead_code)]
fn extend_ttl_is_fine(env: &Env, key: &DataKey) {
    // OK: new API name.
    env.storage().persistent().extend_ttl(key, 100, 1000);
}

#[allow(dead_code)]
fn unrelated_bump(v: &mut semver::Version) {
    // OK: `bump` method exists in many crates. Without a Soroban storage-kind
    // handle upstream, SS014 must not fire.
    v.bump();
}

#[allow(dead_code)]
fn vec_bump(v: &mut Vec<u32>) {
    // OK: unrelated builder-pattern `bump()`, no storage chain.
    v.bump(1);
}
