# Changelog

All notable changes to soroban-scan are documented here.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.3.0] — 2026-04-21

### Added
- **SS013** — `ledger_as_randomness`: flags `env.ledger().timestamp()` and
  `env.ledger().sequence()` used as entropy (modulo, seed, shuffle, RNG
  arguments). Ledger state is deterministic and publicly observable before
  the contract executes, so a sequencer or caller can grind it to
  bias outcomes. Uses a segmented keyword matcher
  (`rand`, `random`, `randomize`, `seed`, `shuffle`, `draw`, `entropy`,
  `rng`, `prng`) rather than substring matching, so DeFi names like
  `withdraw` (contains "draw") and `brand` (contains "rand") do **not**
  fire. Div/mod match is restricted to ledger-on-LHS — patterns like
  `total_supply / env.ledger().sequence()` are per-block averages, not
  entropy primitives, and are ignored.
- **SS014** — `deprecated_bump_api`: flags `.bump(...)` on a Soroban
  storage handle (`.persistent()`, `.temporary()`, `.instance()`). The
  SDK renamed `bump` to `extend_ttl` around v20; the call signature is
  unchanged, so the fix is a mechanical rename. Scoped to the storage
  chain so unrelated `.bump()` methods on `semver::Version`, `Duration`,
  etc. are not flagged.
- `FUNDING.json` (Drips.network) for passive dependency-funding claims
  across Ethereum / Optimism / Filecoin.
- `docs/SCF-BUILD-AWARD-SUBMISSION.md` — full Stellar Community Fund
  Build Award submission draft.

### Changed
- `tests/rules.rs` SARIF shape assertion now expects 14 rules.

### Summary
SS013 + SS014 bring the rule count to **14** (SS001–SS014). All existing
rules and the SARIF / GitHub Action integrations are unchanged.

## [0.2.1] — 2026-04-21

### Changed
- **SS001** re-graded from High to Low, **SS006** re-graded from Low to
  Info after reading [CAP-0066] (Protocol 23, Sep 2025): archived
  persistent entries are auto-restored on `InvokeHostFunctionOp`, so the
  "archive → default → replay" attack model is incorrect. SS001 now
  fires only on the real concern (initialization-flag confusion); SS006
  is an informational nudge. See `docs/BLOG-CAP0066-REGRADE.md` for the
  full rationale.

[CAP-0066]: https://github.com/stellar/stellar-protocol/blob/master/core/cap-0066.md

## [0.2.0] — 2026-04-21

### Added
- SARIF 2.1.0 output (`--format sarif`) — findings load into the GitHub
  Security tab like CodeQL, with inline PR annotations.
- GitHub Action (`action.yml`) — any Soroban repo can add soroban-scan
  to CI in three lines and optionally upload the SARIF.
- Taint-lite heuristic for SS002: `Address` parameters passed as
  `Client::new(env, &x)` are no longer treated as auth candidates.

## [0.1.x] — 2026-04-21

Initial public releases. CLI (`soroban-scan <path>`), 12 rules
(SS001–SS012), fixtures + regression tests, CI (build / test / clippy
-D warnings), MIT license.
