// Fixture for SS016 — each of these initializer shapes is considered safe.
// Parsed by syn; does not need to compile against soroban-sdk.

#[contractimpl]
impl AtomicCtor {
    // GOOD: Soroban-21 atomic constructor runs in the same host op as deploy,
    // so there is no front-run window.
    pub fn __constructor(e: Env, admin: Address) {
        write_admin(&e, &admin);
    }
}

#[contractimpl]
impl AuthedInit {
    // GOOD: `admin.require_auth()` on entry prevents a front-runner from
    // initialising on behalf of the real admin.
    pub fn initialize(e: Env, admin: Address) {
        admin.require_auth();
        if has_admin(&e) {
            panic!("already initialized");
        }
        write_admin(&e, &admin);
    }
}

#[contractimpl]
impl ZeroArgInit {
    // GOOD: initialiser takes no privileged parameter — nothing to hijack.
    pub fn initialize(e: Env) {
        write_version(&e, 1u32);
    }
}
