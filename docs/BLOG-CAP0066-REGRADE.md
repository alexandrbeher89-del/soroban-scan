# When your own scanner is wrong: re-grading SS001 and SS006 after CAP-0066

*Draft post for dev.to / Medium / Stellar Discord #dev-general.*

**TL;DR:** `soroban-scan` v0.2.1 demotes SS001 (`unchecked_storage_get_with_default`) from High to Low and SS006 (`persistent_write_without_extend_ttl`) from Low to Info. The mental model behind the old severity — "archived persistent entries are read back as a default, so an attacker can replay after TTL" — is incorrect on Soroban mainnet post-Protocol 23. CAP-0066 auto-restores archived entries (or refuses to execute the transaction). Contract code can never observe the `unwrap_or` default for a key that was previously written.

This post walks through the mistake, the correction, and why publishing the correction matters more than quietly bumping severities.

## The mistake

`soroban-scan` was grounded in a reading of public Soroban audit reports. A recurring pattern in WatchPug / Halborn / Zellic V12 findings looked like this:

```rust
let used: bool = env
    .storage()
    .persistent()
    .get(&Key::UsedHash(hash))
    .unwrap_or(false);
if used { return Err(Error::AlreadyUsed); }
env.storage().persistent().set(&Key::UsedHash(hash), &true);
```

The intuitive failure model, transplanted from EVM, was: persistent storage has a TTL; if an attacker waits out the TTL, the entry archives, the next read returns the `unwrap_or` default (`false`), and a previously-used signed payload can be replayed. That model drove SS001 to High severity and SS006 (missing `extend_ttl`) to Low.

The model is wrong. It was wrong before Protocol 23, and it is wrong now.

## What Soroban actually does (CAP-0066)

[CAP-0066] activated on Stellar mainnet with Protocol 23 in September 2025. The relevant paragraphs:

> This CAP introduces automatic restoration via `InvokeHostFunctionOp`, where any archived key present in the footprint is automatically restored.

Concretely, a Soroban persistent entry has three states: **live**, **archived**, **evicted**. An archived entry is not a missing entry, and it has never been treated as one. Two things can happen when a contract call touches an archived key:

1. **Footprint declares the key as archived-to-restore.** The Stellar RPC auto-populates this list during simulation. The protocol restores the entry from the archive, charges a restoration fee to the transaction's resource budget, and makes the original value available to the contract. `get` returns the stored value; `unwrap_or` is never reached.
2. **Transaction omits the archived key from the restore list.** The host rejects the invocation at the host-function boundary before any contract code executes.

There is no third branch where contract code runs and observes `unwrap_or`'s default for a previously-written key.

CAP-0066 didn't create this protection. Before Protocol 23 the same invariant held — you just had to call `RestoreFootprintOp` manually before reading. The protocol never had EVM's "absent == zero" storage model. My scanner was encoding a bug class that cannot exist under Soroban's semantics.

Credit where it's due: I ran into this after reading Dan23RR's ["How Soroban's CAP-0066 Killed My LayerZero Finding"][dan23rr], which walks through the same mistake applied to a real Code4rena audit. If you're auditing Soroban with EVM intuition, read that post first.

## What changed in v0.2.1

- **SS001** (`unchecked_storage_get_with_default`): **High → Low.** The rule still fires on the same syntactic pattern, but the rationale is now about *initialization flags*, not replay. `unwrap_or(false)` on an `Initialized` marker lets anyone re-run an initializer if the author never set the flag via a setter. `unwrap_or(Vec::new())` on an authorized-callers list means an uninitialized contract has an empty allow-list, which may or may not be the intended sentinel. These are real concerns, but they are Low-severity code-quality issues, not replay/archival vectors. The rule's `note` field now points at CAP-0066 explicitly so consumers of the JSON/SARIF output don't repeat the old misreading.
- **SS006** (`persistent_write_without_extend_ttl`): **Low → Info.** Missing `extend_ttl` is a gas / UX hint. The next caller who touches an evicted entry pays a restoration fee (and may exceed their tx fee budget), but the stored value is preserved. If the entry is on a hot path, adding `extend_ttl` amortises the rent cost onto the writer. That's worth flagging, but it is not a security finding.
- **Case studies** (`docs/CASE-STUDIES.md`): updated severity distributions for the four public protocols and K2. Raw finding counts are unchanged; the severity column in JSON/SARIF output is new.
- **Rule docstrings**: both rules now link to CAP-0066 and distinguish the debunked model from the real residual concern.

## Why publish this instead of quietly bumping severities

Published security tooling skews heavily toward confirmed findings and hyped severity. The aggregate effect is that consumers of these tools — developers deciding what to fix, auditors triaging a list of hits, bounty programs deciding what to reward — learn a calibration that's too hot. A scanner that cries High on a pattern that can't fire in practice trains its users to ignore High. That's worse than no scanner.

`soroban-scan`'s whole pitch is "catch the patterns that keep showing up in every audit report." That only works if the severities match what audit judges actually rate. SS001 and SS006 did not. The honest correction is more useful to users than a scanner that posts a louder High on the same line.

If you're building a scanner in this ecosystem (or any EVM-to-Soroban port), two habits worth stealing:

1. **Read the CAPs that affect your rules.** CAP-0062, CAP-0057, CAP-0066 rewrote Soroban's storage semantics. They're short and readable. Your rules should cite them when they touch storage-lifecycle concerns.
2. **Test rule severity against audit judges, not against pattern matchers.** If Zellic V12 flagged 57 of these on K2 and all 57 were downgraded to Informational by the judge, your rule is Informational, not High. The judge's severity is ground truth.

## Getting v0.2.1

```bash
cargo install --git https://github.com/alexandrbeher89-del/soroban-scan --tag v0.2.1
```

Or in CI:

```yaml
- uses: alexandrbeher89-del/soroban-scan@v0.2.1
  with:
    path: ./contracts
    fail-on: high
```

Issues, false positives, and rule suggestions: <https://github.com/alexandrbeher89-del/soroban-scan/issues>.

[CAP-0066]: https://github.com/stellar/stellar-protocol/blob/master/core/cap-0066.md
[dan23rr]: https://medium.com/@daniel.culotta_89017/layerzero-post-medium-md-ab5a8fddba41
