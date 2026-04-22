// Fixture for SS013 — ledger timestamp / sequence used as entropy.
// None of this actually needs to compile against soroban-sdk; we only parse
// with syn. The shapes below are what we flag in real contracts.

#[allow(dead_code)]
fn pick_winner_bad(env: &Env) -> u64 {
    // BAD: timestamp mod — trivially grindable by sequencer.
    env.ledger().timestamp() % 100
}

#[allow(dead_code)]
fn shuffle_index_bad(env: &Env) -> u32 {
    // BAD: sequence mod — deterministic / public before contract runs.
    (env.ledger().sequence() as u32) % 7
}

#[allow(dead_code)]
fn make_seed_bad(env: &Env) -> u64 {
    // BAD: ledger timestamp fed into a function named like a seed primitive.
    seed_from(env.ledger().timestamp())
}

#[allow(dead_code)]
fn shuffle_bad(env: &Env, deck: &mut [u8]) {
    // BAD: ledger sequence fed into a shuffle method.
    deck.shuffle_with(env.ledger().sequence());
}

// --- non-matching patterns (should NOT fire) -------------------------------

#[allow(dead_code)]
fn deadline_check(env: &Env, deadline: u64) -> bool {
    // OK: comparison, not entropy.
    env.ledger().timestamp() < deadline
}

#[allow(dead_code)]
fn age_math(env: &Env, start: u64) -> u64 {
    // OK: subtraction of two timestamps, not entropy.
    env.ledger().timestamp() - start
}
