# Stellar Discord / #dev-general short post

*Single message, <= 2000 chars (Discord limit). Low-key, informational, no ask.*

---

Hey folks — I built **soroban-scan**, a small open-source static analyzer
for Soroban contracts. Point it at a directory of `.rs` files, it parses
them through `syn` and fires on audit-grounded bug patterns (TTL expiry,
missing `require_auth`, integer narrowing, unbounded iteration, CEI
violations, etc.).

v0.1.2 ships 12 rules, all grounded in public audit findings (Code4rena K2,
WatchPug rounds, Zellic V12). Single binary, sub-200ms scans on 30kLoC.

Ran it on four public protocols as a sanity check — reproducible numbers in
the repo:

- Blend:    49 findings
- Soroswap: 39 findings
- DeFindex: 99 findings
- Phoenix: 129 findings

Most are "triage me" not "this is broken" — by design; the tool is a first
pass, not a verdict. FPs get filed as issues with a minimal fixture and the
rule gets tuned.

Repo: https://github.com/alexandrbeher89-del/soroban-scan
Case studies: https://github.com/alexandrbeher89-del/soroban-scan/blob/devin/init-soroban-scan/docs/CASE-STUDIES.md

Feedback / rule requests / false-positive reports very welcome.
