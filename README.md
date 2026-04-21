# soroban-scan

**Static analyzer for Soroban (Stellar) smart contracts written in Rust.**
Detects common security bug classes observed in real Soroban audit reports
(K2 / WatchPug / Halborn / Zellic V12 on Code4rena).

Where `cargo clippy` is stylistic and `cargo audit` is supply-chain,
**soroban-scan is audit-oriented**: it fires on patterns that repeatedly show
up in signed audit reports as *Medium* or *High* severity findings, so you
can reproduce a solid first pass before paying for a human audit.

> Status: **v0.1 — early, useful, best-effort**. Heuristic-based: expect
> false positives; triage reported findings by hand.

---

## Install

```bash
cargo install --git https://github.com/alexandrbeher89-del/soroban-scan
```

or from source:

```bash
git clone https://github.com/alexandrbeher89-del/soroban-scan
cd soroban-scan
cargo build --release
./target/release/soroban-scan --help
```

## Usage

```bash
soroban-scan path/to/contracts
soroban-scan path/to/contracts --format json
soroban-scan path/to/contracts --skip SS010,SS005   # disable specific rules
soroban-scan path/to/contracts --fail-on medium     # non-zero exit on M/H
```

Use it in CI to regression-guard new code:

```yaml
- name: Static audit (soroban-scan)
  run: |
    cargo install --git https://github.com/alexandrbeher89-del/soroban-scan
    soroban-scan ./contracts --fail-on high
```

## Rules

| ID     | Severity | Rule                                    | Bug class                                                      |
| ------ | -------- | --------------------------------------- | -------------------------------------------------------------- |
| SS001  | High     | `unchecked_storage_get_with_default`    | `persistent().get().unwrap_or(...)` hides archived/expired entries behind a silent default. |
| SS002  | High     | `missing_require_auth`                  | Public contract function takes an `Address` parameter but never authenticates it.             |
| SS003  | Medium   | `unbounded_loop_over_storage_collection`| `for _ in storage_vec.iter()` without a length guard — DoS risk on 100M CPU budget.           |
| SS004  | Medium   | `state_write_after_external_call`       | Storage mutation follows an external `invoke_contract` / client call in the same function (CEI). |
| SS005  | Med/Low  | `wide_to_narrow_downcast`               | `.to_u128()` / `as u64` on wider integers may truncate silently in financial math.            |
| SS006  | Low      | `persistent_write_without_extend_ttl`   | `persistent().set(...)` without a matching `extend_ttl` — entries archive and readers degrade. |
| SS007  | High/Med | `storage_enum_key_collision`            | `#[contracttype]` enum variants share a discriminant or collide modulo case.                  |
| SS008  | Low      | `bitmap_or_without_clear_mask`          | `x \|= mask` on a bitmap without clearing the target bits first; leaves stale flags.          |
| SS009  | Low      | `division_before_multiplication`        | `(a / b) * c` loses precision — rearrange to `(a * c) / b`.                                    |
| SS010  | Info     | `missing_event_on_state_change`         | State-changing function never calls `env.events().publish(...)`.                              |
| SS011  | Medium   | `partial_auth_coverage`                 | Function auths some but not all `Address` parameters.                                         |
| SS012  | Low      | `panic_in_contract_function`            | `unwrap()` / `expect()` / `panic!()` inside `#[contractimpl]` — prefer typed `Error`.         |

Each rule includes a fixture under `tests/fixtures/` and a regression test in
`tests/rules.rs`.

## Why this exists

The Soroban ecosystem is young and has **no Slither-equivalent**. Reviewing
the public Code4rena K2 audit (April 2026, $135k pool, Aave-V3 fork on Soroban),
the top recurring bug classes across five audit rounds (Halborn + WatchPug ×4)
and ~500 AI-generated findings from Zellic V12 were:

- TTL-expiry collapsed into valid default (SS001 / SS006)
- Missing or partial auth on `Address` parameters (SS002 / SS011)
- Bitmap desync in user-config storage (SS008)
- U256 → narrower-int truncation (SS005)
- Unbounded loops over reserve lists (SS003)
- CEI violations around swap adapters (SS004)

These are exactly the patterns soroban-scan fires on. A 10-second CLI run
shouldn't replace an audit, but it should catch the patterns that keep
showing up in every audit report.

## Example: scanning the K2 audit repo

```text
$ soroban-scan ./contracts
soroban-scan: 121 files scanned, 369 findings (205 high, 49 med, 115 low, 0 info)

HIGH [SS001] interest-rate-strategy/src/storage.rs:86:52
    base_variable_borrow_rate: env.storage().instance().get(&BASE_VAR_RATE).unwrap_or(0),
    hint: Use .expect()/.ok_or(...)? or explicitly handle the None case; otherwise
    a missing (e.g. archived-then-restored-as-default, or never-initialized) entry
    is indistinguishable from a legitimate empty value.

HIGH [SS002] debt-token/src/contract.rs:46:12
    pub fn transfer(
    hint: Parameter(s) never auth'd: from, to. Add `from.require_auth()` early in
    the function, or document why the caller is trusted.
...
```

(Most findings require triage. That's by design: soroban-scan is a first
pass, not a verdict.)

## Contributing

PRs welcome. To add a new rule:

1. Add `src/rules/ssNNN_your_rule.rs` implementing `Rule`.
2. Register it in `src/rules/mod.rs::all_rules()`.
3. Add a positive fixture at `tests/fixtures/ssNNN_bad.rs` and a
   `#[test]` in `tests/rules.rs`.
4. Run `cargo test` and `cargo clippy -- -D warnings`.

## Supporting the project

If `soroban-scan` saved you audit time or caught a real bug, consider:

- **Tip jar (EVM — ETH / Arbitrum / Base / Polygon / BSC):**
  `0x04dd1AcaC0a5C498A8f26fcc745dc573B4EDcBFa`
- Stellar Community Fund (SCF) builder grants are being pursued —
  endorsements welcome.
- Open an issue with a false positive / negative you'd like handled.

## License

MIT. See [LICENSE](./LICENSE).
