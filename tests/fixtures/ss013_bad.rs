// Fixture for SS013 — ledger timestamp used as entropy via modulo.
// Parsed by syn; does not need to compile against soroban-sdk.

#[allow(dead_code)]
fn pick_winner_bad(env: &Env) -> u64 {
    // BAD: timestamp mod — trivially grindable by sequencer.
    env.ledger().timestamp() % 100
}
