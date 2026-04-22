// Negative fixture for SS013 — legitimate uses of env.ledger().timestamp()
// that must NOT fire (comparisons, subtractions, passed to a non-entropy
// function). Parsed by syn; does not need to compile.

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

#[allow(dead_code)]
fn store_last_seen(env: &Env) -> u64 {
    // OK: timestamp passed to a function that is NOT named like a random /
    // seed / shuffle primitive.
    record_observation(env.ledger().timestamp())
}

#[allow(dead_code)]
fn withdraw_with_deadline(pool: &Pool, from: &Address, amount: i128, env: &Env) {
    // OK: `withdraw` contains the substring "draw" but is NOT a randomness
    // primitive. Must not be flagged.
    pool.withdraw(from, amount, env.ledger().timestamp() + 300);
}

#[allow(dead_code)]
fn brand_check(env: &Env) -> u64 {
    // OK: `brand` contains the substring "rand" but is NOT a randomness
    // primitive. Must not be flagged.
    brand(env.ledger().timestamp())
}

#[allow(dead_code)]
fn per_block_average(total: u64, env: &Env) -> u64 {
    // OK: ledger value on the RHS of `/` — this is a per-block average, not
    // an entropy primitive. Flagging it would misformat the note as
    // `ts % N` when the actual shape is `N / ts`.
    total / env.ledger().sequence() as u64
}
