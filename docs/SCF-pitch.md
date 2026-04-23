# SCF Build Award — soroban-scan (draft)

This is a **draft pitch** for a Stellar Community Fund Build Award. It is not
yet submitted. The final application form lives on
<https://communityfund.stellar.org/>; this document is the raw content.

---

## Project name

**soroban-scan** — an open-source, audit-oriented static analyzer for Soroban
smart contracts.

## One-line summary

`cargo clippy` for Soroban auditors: detects the security bug classes that
repeatedly show up in public Soroban audit reports.

## Problem

The Soroban ecosystem has shipped a growing number of DeFi and RWA protocols
(K2, Blend, Aqua, Soroswap, DeFindex, Phoenix, …) but has **no Slither-equivalent**:
no widely-used static analyzer tuned to the bug classes auditors actually find.
Each new audit engagement therefore starts from the same manual checklist: TTL
expiry defaults, missing `require_auth`, bitmap desync, U256 truncation, CEI
violations around `invoke_contract`, etc.

This hurts:

- **Builders** pay for every audit hour spent re-surfacing patterns a tool
  could flag in 10 seconds.
- **Auditors** burn budget on pattern-matching instead of on reasoning.
- **SCF/Stellar** end up with protocols that ship with known bug classes
  simply because the community doesn't have a shared pre-audit gate.

## Solution

`soroban-scan` is a single-binary Rust CLI that:

1. Parses Soroban contracts via `syn`.
2. Runs a suite of audit-oriented rules (16 in v0.4.0, growing).
3. Reports findings with file, line, snippet, severity, and remediation hint.
4. Exits non-zero on configurable severity thresholds, so it drops cleanly
   into CI.

Each rule is grounded in a **real audit finding** from a public Soroban audit
(Halborn, WatchPug, Zellic V12 on K2; and prior Blend / Aqua / Soroswap
reports). Nothing is invented — the rules target patterns that have already
been paid for in bounties.

## v0.4.0 status (as of application)

- Repo: <https://github.com/alexandrbeher89-del/soroban-scan>
- License: MIT
- 16 rules implemented (SS001–SS016), each with fixture + regression test.
- SARIF 2.1.0 output + reusable GitHub Action at repo root.
- x402-paywalled HTTP API (`api/`) wrapping the scanner — USDC-on-Base per
  request, testnet deployment on Fly.io, mainnet flip is one env var.
- GitHub Actions CI green (build + test + clippy `-D warnings`).
- Smoke test on the April-2026 Code4rena K2 repo (~37k LoC, 121 files):
  369 raw findings in 2.1 s wall-clock.

## What the grant buys

Budget request: **$5,000 USDC** (Build Award tier 1), 6-8 week effort.

Deliverables:

1. **21 additional rules** (target: 37 total) covering the remaining
   high-frequency classes from public Soroban audits: cross-contract
   reentrancy patterns, archive/restore edge-cases, fee-math rounding,
   price-oracle staleness, admin-key lifecycle, storage-key prefix collisions,
   `extend_ttl` arithmetic, `symbol_short!` truncation, SorobanEnv trust
   boundary leaks.
2. **Per-rule test fixture + reproduction of the original audit finding**, so
   each rule links back to the report that justified it.
3. **SARIF output format** for GitHub Code Scanning integration (one-line
   drop-in for any Soroban repo's Actions workflow).
4. **Docs site** with rule catalogue, severity rationale, and contribution
   guide.
5. **Three protocol case studies** — run the tool against three shipped SCF-funded
   protocols (with maintainer consent), share the raw report and confirm
   remediations land upstream.

## Why me / why now

- The tool already exists at v0.4.0, funded on a personal compute budget. The
  grant accelerates from 16 → 37 rules and funds integration work, not
  speculative R&D.
- I'm actively tracking public Soroban audits (K2 round 5 is live on
  Code4rena as of this writing), so each new audit cycle is feedstock for new
  rules.
- Every rule ships with a test fixture that reproduces the originating audit
  finding — results are auditable, not vibes.

## Payout

USDC on Stellar (Issuer: Circle, `USDC`). Project wallet address will be
provided in the application form; all grant funds are spent on compute,
domain, and contributor bounties, not self-income.

## Traction signals (to update before submission)

- [ ] GitHub stars ≥ 20
- [ ] At least one outside contributor (PR merged)
- [ ] Named adoption by one SCF-funded protocol in their CI pipeline
- [ ] Public reference from one credentialed auditor (Halborn / WatchPug /
      Runtime Verification / OpenZeppelin / Zellic)

## Risks / open questions

- **False positives**: heuristic rules will fire on legitimate code. Mitigation:
  per-rule suppression via `--skip`, rule-level severity, explicit "triage
  expected" framing in docs.
- **Soroban language evolution**: SDK changes (e.g. `Storage` API revamps) break
  textual heuristics. Mitigation: rules use syntactic AST matching, not regex;
  rule tests pin against the SDK version of the originating audit.
- **Overlap with commercial tools**: Certora / Runtime Verification may offer
  semantic analysis for Soroban in the future. soroban-scan is explicitly
  positioned as the **free, fast, first-pass** tier, complementary to
  deep semantic tooling.

---

*This document is a working draft. Comments welcome via GitHub issues.*
