# Case studies — soroban-scan on public Soroban protocols

This page reports what `soroban-scan` v0.2.1 finds when pointed at several
well-known open-source Soroban codebases. It is a **triage exercise**, not an
audit: findings are candidates for review, not proven vulnerabilities.

> **v0.2.1 severity re-grading (CAP-0066):** SS001 was demoted High → Low and
> SS006 was demoted Low → Info after re-reading [CAP-0066]. Archived
> persistent entries are auto-restored by the host, so neither rule
> corresponds to a state-integrity attack. SS001 now fires on the real
> concern (initialization-flag confusion) and SS006 is a gas/UX hint. The
> raw finding counts below are unchanged; the severity column in JSON /
> SARIF output is new. All totals below are fresh scans with v0.2.1, so
> they also pick up any upstream noise reductions shipped since the last
> case-study refresh (e.g. v0.1.2's SS002 taint filter); the bucket to
> compare across versions is therefore *the scanner version*, not the
> severity schema alone.
>
> [CAP-0066]: https://github.com/stellar/stellar-protocol/blob/master/core/cap-0066.md

Every number here is reproducible. Commands are listed per protocol. If you
want to reproduce, clone the protocol at the same commit and run the command
shown — you should get the same raw totals (some noise may differ if the
upstream repo has been rebased).

Starting with v0.1.1, soroban-scan skips items inside `#[cfg(test)]` modules
by default, which removes the bulk of the previous-version noise on projects
that keep tests co-located with implementation (Blend, DeFindex).

## TL;DR

| Protocol                  | Rust LoC | Files | Scan time | Raw findings |
| ------------------------- | -------- | ----- | --------- | ------------ |
| Code4rena K2 (April 2026) |  ~37k    |  121  |  2.1 s    |  369         |
| Blend protocol            |  29.7k   |   80  |  0.13 s   |   49         |
| Soroswap core             |  13.9k   |   74  |  0.07 s   |   39         |
| DeFindex                  |  32.0k   |  138  |  0.16 s   |   99         |
| Phoenix protocol          |  33.0k   |   56  |  0.06 s   |  129         |

soroban-scan is **fast** — <200 ms on 30k LoC — which is what makes it
practical to keep in CI.

---

## Blend (lending / liquidation protocol)

- Repo: <https://github.com/blend-capital/blend-contracts>
- Command: `soroban-scan /path/to/blend-contracts`

Totals: 49 findings (25 high, 20 low, 4 info).

| Rule  | Count | Triage notes                                                                                                     |
| ----- | ----- | ---------------------------------------------------------------------------------------------------------------- |
| SS001 |   1   | `.unwrap_or(false)` in `emitter/src/storage.rs:190` — worth a look; if the entry is persistent, a false default could hide a legitimate state. |
| SS002 |  25   | Mostly token-interface methods (`transfer`, `approve`) where auth is handled inside the wrapped token contract. Expect ~60% FPR here, but the remaining ~40% are plausible starting points. |
| SS005 |  19   | U128 → narrower casts in rate and oracle math. Blend uses fixed-point extensively; each deserves a line-by-line check that the cast is bounded. |
| SS006 |   4   | Persistent writes without `extend_ttl` — worth reading, these survive the v0.1.1 test-code filter.              |

Blend has had three public audits and mostly shows up as background noise
plus a small handful of SS005/SS006 items worth reviewing.

## Soroswap core (AMM)

- Repo: <https://github.com/soroswap/core>
- Command: `soroban-scan /path/to/soroswap/core`

Totals: 39 findings (26 high, 13 low, 0 info).

| Rule  | Count | Triage notes                                                                                        |
| ----- | ----- | --------------------------------------------------------------------------------------------------- |
| SS001 |   2   | `unwrap_or` in storage getters — check if they gate an archive-vs-uninitialized distinction.        |
| SS002 |  27   | Most flags are on `token_a: Address` / `token_b: Address` / `factory: Address` — these are asset/contract identifiers, not user auth targets. Confirmed false positives. The recipient `to: Address` is correctly `require_auth()`'d throughout. |
| SS005 |   4   | Integer conversions in reserve math. Each should be bounded; needs spot-check.                      |
| SS012 |   7   | `.unwrap()` on storage reads inside contract methods — would gain from `.ok_or(ErrCode::...)?`.     |

## DeFindex (vault / strategy framework)

- Repo: <https://github.com/paltalabs/defindex>
- Command: `soroban-scan /path/to/defindex`

Totals: 99 findings (39 high, 48 low, 12 info).

| Rule  | Count | Triage notes                                                                                  |
| ----- | ----- | --------------------------------------------------------------------------------------------- |
| SS001 |   7   | Worth reading. DeFindex has vault-strategy routing where a silent default could alter a user's share accounting. |
| SS002 |  39   | Strategy and hodl contracts — similar triage pattern to Soroswap. Spot-check for genuine auth gaps in strategy entry points. |
| SS005 |  31   | DeFindex does many i128/u128 conversions in strategy adapters. Each should be bounded or use `try_into()`. |
| SS006 |  12   | Real-looking persistent writes without `extend_ttl`.                                          |
| SS012 |  10   | `.unwrap()` / `panic!` inside `#[contractimpl]` entries — each can turn into a DoS.           |

## Phoenix (DEX / pool / farm)

- Repo: <https://github.com/Phoenix-Protocol-Group/phoenix-contracts>
- Command: `soroban-scan /path/to/phoenix-contracts`

Totals: 129 findings (32 high, 9 medium, 76 low, 12 info).

| Rule  | Count | Triage notes                                                                                       |
| ----- | ----- | -------------------------------------------------------------------------------------------------- |
| SS001 |  27   | Highest SS001 count of any protocol scanned. Worth an hour of human time.                          |
| SS002 |  35   | Mixed — some are clearly asset-Address false positives, some look like missing auth on staker/farmer entry points. |
| SS005 |  52   | Same pattern: i128/u128 conversions. Check for bounded input domains.                              |
| SS006 |  12   | Persistent writes without `extend_ttl`.                                                            |
| SS009 |   2   | `(a / b) * c` precision — worth a read if these touch payouts.                                    |
| SS012 |   4   | Unwrap inside contract methods.                                                                    |

## Code4rena K2 (April 2026)

- Repo: <https://github.com/code-423n4/2026-04-k2> (private; only visible to C4 wardens)
- Command: `soroban-scan /path/to/k2/contracts`

Totals: 369 findings (205 high, 49 medium, 115 low).

| Rule  | Count | Note                                                                                               |
| ----- | ----- | -------------------------------------------------------------------------------------------------- |
| SS001 |  57   | TTL defaults — K2 was flagged heavily on this class by Zellic V12.                                 |
| SS002 | 148   | K2's Aave-V3 port threads `Address` through many internal helpers. Significant FP share expected.  |
| SS005 | 157   | Extensive U256 / fixed-point math — most are plausible starting points.                            |
| SS006 |   6   | Persistent entries without `extend_ttl`.                                                           |
| SS011 |   1   | Partial auth coverage — one helper auths 1 of 2 Address params.                                    |

Because K2 was audited five times already (Halborn, WatchPug ×4, Zellic V12),
most of these are expected to have been triaged during the audit process.
Nonetheless, 1 new Medium from this class of tool on a five-times-audited
codebase is a reasonable expectation.

## Known false-positive patterns (v0.1)

From these case studies, the single dominant FP source is:

- **`Address` parameters that are not user identities** — token addresses,
  factory addresses, reserve addresses. Soroban's `Address` type is
  polymorphic (used for users, for contracts, and for issued assets). SS002
  treats all of them as potential auth targets. Planned v0.2 fix: light taint
  heuristic — an `Address` passed directly to `Client::new(&env, &x)` or
  `token::Client::new(…)` is treated as an asset/contract reference and
  excluded from the SS002 "must call require_auth" check.

The `#[cfg(test)]` FPs that dominated the v0.1.0 output on Blend are fixed in
v0.1.1 — tests are skipped by default. Pass `--include-tests` to opt back in.

## Try it yourself

```bash
cargo install --git https://github.com/alexandrbeher89-del/soroban-scan
git clone --depth 1 https://github.com/blend-capital/blend-contracts
soroban-scan blend-contracts
soroban-scan blend-contracts --format json | jq '.[] | select(.severity=="high")'
```

Found a new false positive? Open an issue with the minimal fixture — we'll
add a regression test and tune the rule.
