# SCF Interest Form — refreshed content (2026-04-23)

The original Interest Form for `soroban-scan` was submitted on 2026-04-21 with
the v0.2.1 / 12-rule snapshot. Since then the project has shipped v0.4.0
(16 rules), launched a pay-per-call HTTP API live on Base mainnet, and
tagged six releases in the repo CHANGELOG. This document is the **refreshed
content** to paste into a second Interest Form submission at
<https://communityfund.stellar.org/> so the reviewer sees the current state.

Form source of truth: <https://communityfund.stellar.org/>. Fields below
match the labels visible on the SCF Interest Form as of Round #43.

---

## Project name

**soroban-scan**

## One-line summary (≤ 140 chars)

> `cargo clippy` for Soroban auditors: a Rust CLI, GitHub Action, and x402
> HTTP API that flag 16 audit-proven Soroban bug classes.

## Category

Developer Tools / Security / Public Goods (sub-select the closest available
option; e.g. *"Developer Tooling"* or *"Infrastructure"*).

## Repository

<https://github.com/alexandrbeher89-del/soroban-scan> — MIT licensed, public,
CI green on every push.

Release tags: v0.1.2, v0.2.0, v0.2.1, v0.4.0. `Cargo.toml` version: `0.4.0`.

## Team

Solo builder — one contributor. Active on GitHub under
`alexandrbeher89-del`. No employees, no co-founders, no external funding.

## Stage

**Shipped, in use.** The CLI, GitHub Action, SARIF integration, and x402 API
are all live and exercised in CI on every commit.

## Problem

The Soroban ecosystem has 30+ production protocols (Blend, Soroswap,
DeFindex, Phoenix, Aqua, K2, …) and **no widely-used static analyzer**. Every
new audit engagement manually re-discovers the same bug classes:

- TTL expiry / CAP-0066 restoration edge-cases
- Missing `require_auth` on sensitive entry points
- i128 → u64 downcast truncation
- `invoke_contract` reentrancy / CEI violations
- Storage-key enum collisions, bitmap desync, `f64` math breaking consensus,
  unauthenticated `initialize()` letting deploys be front-run, etc.

Each of these appears repeatedly in published audit reports from Halborn,
Certora, WatchPug, Zellic V12, OtterSec. Builders pay for every audit hour
spent re-surfacing patterns a tool could flag in 10 seconds.

## Solution

`soroban-scan` is a single-binary Rust CLI that parses Soroban contracts with
`syn`, runs 16 audit-oriented rules (SS001–SS016), and emits findings as
human / JSON / SARIF 2.1.0.

- **CLI**: `soroban-scan path/ [--format sarif] [--fail-on high]`
- **GitHub Action**: 3-line composite action (`action.yml`) that feeds SARIF
  straight into a repo's *Security → Code scanning* tab.
- **Pay-per-call HTTP API**: `api/` wraps the scanner behind an
  [x402](https://x402.org) paywall. Each request settles real USDC on Base
  directly to the project wallet. Live at
  <https://soroban-scan-api-qhldjpbq.fly.dev/>.

Every rule is grounded in a *published* Soroban audit finding — the rule
docstring and test fixture cite the originating report. Nothing is invented.

## Current state (as of 2026-04-23)

- **v0.4.0** tagged; 16 rules (SS001–SS016), each with a `tests/fixtures/`
  reproduction.
- `cargo test --release`: 15/15 green. `cargo clippy --all-targets -- -D
  warnings`: clean. CI (dtolnay/rust-toolchain@stable) green on every push.
- **x402 API live on Base mainnet** via the permissionless OpenX402
  facilitator. `POST /scan` = $0.01 USDC, `POST /scan/repo` = $0.05 USDC.
- 4 reproducible case-study scans on Blend / Soroswap / DeFindex / Phoenix
  under [`docs/CASE-STUDIES.md`](./CASE-STUDIES.md).
- [CAP-0066 severity re-grading writeup](./BLOG-CAP0066-REGRADE.md) — public
  self-correction after SDF's auto-restore change landed in Protocol 23.
- Full Build Award submission pre-written at
  [`SCF-BUILD-AWARD-SUBMISSION.md`](./SCF-BUILD-AWARD-SUBMISSION.md).

## Why a grant

A Build Award accelerates the rule catalogue from **16 → 37 rules** (+21
new), funds a docs site with the rule catalogue, SARIF schema pinning, a
tagged `v1.0.0` GitHub Marketplace release, and three upstreamed protocol
case studies with SCF-funded maintainers.

Estimated ask: **$10,000 USDC-equivalent in XLM** (Build Award entry tier),
milestone-gated across five tranches. Full budget breakdown is in the
pre-written Build Award submission.

## Why me / why now

- The tool is already at v0.4.0 on a personal compute budget. The grant
  funds rule work + integration + documentation, not speculative R&D.
- Soroban audit cadence is accelerating (K2 R5 on Code4rena as of this
  writing) — each new audit cycle is feedstock for new rules.
- The x402-paywalled HTTP API demonstrates an alternative, direct
  monetization path that keeps the core CLI **free and open source** while
  giving the project a sustainable long-term revenue mechanism.

## Payout

XLM on Stellar mainnet, address:
`GBMNXH6ZIOQ7L6FGI5MMXXM3QBL646HV57JGD5LRJZ3JFF6EI4PRHGHG`.

## Traction signals

- 6 tagged releases (v0.1.x → v0.4.0) over three weeks.
- 8 PRs merged in the week leading up to this resubmission.
- x402 API live on Base mainnet — real-money paywall in production.
- Case studies published on four shipped Soroban protocols.
- Public CAP-0066 re-grading post demonstrating audit-level rigor.

## Risks / open questions

- **False positives**: heuristic rules will fire on legitimate code.
  Mitigation: per-rule suppression via `--skip`, severity tiers, explicit
  "triage expected" framing in docs.
- **Soroban SDK evolution**: SDK revamps can break textual heuristics.
  Mitigation: rules use syntactic AST matching (not regex); each rule's
  fixture pins against the SDK version of the originating audit.
- **Overlap with commercial tools**: Certora / Runtime Verification may ship
  Soroban coverage. Mitigation: `soroban-scan` is *audit-oriented heuristics*,
  not formal verification — the niche is "first pass before paying for an
  audit", not "replacement for an audit".

---

## Submission checklist

Before pasting into the form:

- [ ] Confirm the form still accepts a *refreshed* Interest Form from a
      project that already has one under review (or ask SCF liaison in
      `#scf-general` Discord before resubmitting to avoid duplicate-entry
      flagging).
- [ ] Copy the sections above into the corresponding fields.
- [ ] Attach / link the live x402 API URL as a demo.
- [ ] Link this file's raw URL
      (`https://github.com/alexandrbeher89-del/soroban-scan/blob/master/docs/SCF-INTEREST-FORM-RESUBMIT.md`)
      as an appendix, if the form allows a "supporting link".
