# Contributing to soroban-scan

Thanks for the interest. soroban-scan is a small project with clear rules for
what goes in and what doesn't.

## What belongs here

- **New rules** grounded in a real public audit finding (Code4rena, Sherlock,
  Cantina, CodeHawks, Halborn, WatchPug, Zellic, Trail of Bits, OpenZeppelin,
  Runtime Verification, and similar).
- **False-positive fixes** with a minimal reproducing fixture + regression
  test that asserts the rule does *not* fire on that fixture.
- **Output formats** (SARIF, GitLab code-quality JSON, JUnit, etc.) that
  integrate with existing CI dashboards.
- **Performance improvements** — the tool's one durable moat is speed, so
  regressions below 200 ms per 30 kLoC are tracked closely.

## What does not belong here

- Rules that are not backed by a public audit finding. Integrity-first —
  speculative patterns become false-positive engines.
- Rules targeting non-Soroban smart-contract languages. (`solana-scan` is a
  great future project; out of scope here.)
- Runtime / dynamic analysis. soroban-scan is strictly static. Use
  `cargo fuzz` and Soroban's native test runner for runtime concerns.

## Adding a new rule

1. Pick the next `SSNNN` id. Create `src/rules/ssNNN_short_name.rs`.
2. Implement the `Rule` trait. Keep it under 80 lines if possible — the
   visitor pattern from syn makes this feasible.
3. Register the rule in `src/rules/mod.rs::all_rules()`.
4. Add a positive fixture: `tests/fixtures/ssNNN_bad.rs`.
5. Add an integration test in `tests/rules.rs` that asserts the rule fires
   exactly once on the fixture.
6. If applicable, add a negative fixture (`ssNNN_good.rs`) and a test that
   asserts the rule does not fire.
7. Update `docs/CASE-STUDIES.md` if the rule fires on one of the tracked
   protocols (opt-in).
8. Update the "Rules" section of `README.md`.

## Running the test suite

```bash
cargo test --release
cargo clippy --all-targets --release -- -D warnings
cargo fmt --check
```

All three must be clean on your branch before submitting a PR — CI enforces
the same.

## Triage conventions for fixtures

- **Positive fixture**: minimal Rust that compiles in isolation and
  reproduces the bug class. No `fn main`. Use a `fn` inside a module if
  needed to satisfy the parser; the scanner doesn't require syntactic
  correctness beyond what `syn::parse_file` accepts.
- **Negative fixture**: the same shape, with the bug fixed. Proves the rule
  isn't "match everything".

## License

By contributing, you agree that your contribution is licensed under the MIT
License (matching the project's LICENSE file).
