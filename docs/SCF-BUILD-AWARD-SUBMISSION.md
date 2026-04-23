# SCF Build Award — full submission draft (soroban-scan)

Status: **Draft, pre-invite.** The SCF Interest Form for `soroban-scan` was submitted
on 2026-04-21 (Round #43 Build window). This document is the pre-written full
Build Award submission to be pasted into the next-stage form if/when the Interest
Form is accepted.

Form source of truth: <https://communityfund.stellar.org/>.
Companion doc: [SCF-pitch.md](./SCF-pitch.md) (high-level narrative).

---

## 1. Project summary (≤ 250 chars)

> `soroban-scan` is a free, open-source static analyzer for Soroban smart contracts:
> a Rust CLI + GitHub Action that flags 16 audit-proven bug classes (missing
> `require_auth`, unchecked storage, TTL/CAP-0066, i128→u64 truncation, ledger
> entropy, float math, unauthenticated initializers, etc.) and emits SARIF 2.1.0
> straight into GitHub Security.

## 2. Problem statement

The Soroban ecosystem has 30+ production protocols (Blend, Soroswap, DeFindex,
Phoenix, Aqua, …) and zero widely-used static analyzer. Each audit manually
re-discovers the same class of bugs:

- TTL expiry / CAP-0066 restoration edge-cases
- Missing `require_auth` on sensitive entry points
- i128 → u64 downcast truncation
- `invoke_contract` reentrancy / CEI violations
- Storage-key enum collisions and prefix collisions
- Bitmap OR without mask (desync)
- Division-before-multiplication / rounding drift
- Unchecked `storage.get().unwrap()` panics
- Unbounded `Vec` iteration (gas / TTL blow-up)
- Partial `require_auth` when multiple addresses are in scope
- Admin-lifecycle event emission gaps
- Panic / `unwrap` inside public entry points

Every one of these is from a **published** Soroban audit report (Halborn, Certora,
WatchPug, Zellic V12, OtterSec). soroban-scan automates the first pass so human
auditors start at line-item review instead of at pattern-matching.

## 3. Solution

### What exists today (v0.4.0)

- Rust CLI, single binary: `soroban-scan <path> [--format human|json|sarif] [--skip …] [--fail-on high|medium|low|any]`.
- 16 rules (SS001–SS016), each with a fixture that reproduces the originating
  audit finding.
- Output formats: human, JSON, SARIF 2.1.0.
- Companion composite GitHub Action at repo root (`action.yml`) — 3-line drop-in
  for any Soroban repo's CI pipeline.
- Pay-per-call HTTP API (`api/`) wrapping the scanner with an x402 paywall —
  clients pay USDC on Base per request, payments settle directly to the
  project wallet. Currently deployed on a Base-Sepolia testnet facilitator
  for shape validation; mainnet flip is a one-env-var change.
- MIT licensed. CI green. 6 releases in CHANGELOG (v0.1.x → v0.4.0). 4
  reproducible case-study scans on Blend / Soroswap / DeFindex / Phoenix.
- CAP-0066 severity re-grading writeup (public self-correction after SDF's
  auto-restore change landed in Protocol 23).

Repo: <https://github.com/alexandrbeher89-del/soroban-scan>
Case studies: [docs/CASE-STUDIES.md](./CASE-STUDIES.md)
CAP-0066 regrading: [docs/BLOG-CAP0066-REGRADE.md](./BLOG-CAP0066-REGRADE.md)

### What the grant delivers

A production-ready, community-endorsed static-analysis layer for Soroban with
**37 rules** (16 existing + 21 new), inline GitHub Code Scanning integration,
a documentation site, and three upstreamed protocol case studies.

## 4. Budget

**Total request: $10,000 USDC-equivalent in XLM** (Build Award entry tier).

Milestone-based tranches:

| # | Milestone | % | USD | Acceptance criteria |
|---|-----------|---|-----|---------------------|
| 0 | Kickoff (grant acceptance) | 10% | 1,000 | — |
| 1 | +7 rules (SS017–SS023) with fixtures + tests | 20% | 2,000 | PR merged to `master`; CI green; each rule links to originating audit report in its doc block |
| 2 | +9 rules (SS024–SS032) + rule catalogue site | 25% | 2,500 | `https://alexandrbeher89-del.github.io/soroban-scan/` live; catalogue auto-generated from rule metadata |
| 3 | +5 rules (SS033–SS037) + SARIF schema pinning + GitHub Action v1.0 release | 20% | 2,000 | Tagged `v1.0.0` release; Action consumable from Marketplace; third-party Soroban repo PR opened (even if not merged) |
| 4 | 3 upstream case-study reports with SCF-funded protocol maintainers (consent-gated) | 15% | 1,500 | 3 reports published under `docs/CASE-STUDIES/`; at least one remediation PR opened upstream |
| 5 | Final writeup + grant report to SCF | 10% | 1,000 | Public blog post summarizing findings, rule coverage map, adoption metrics |

Total: 100% / $10,000.

**What the money is spent on** (itemized):

| Item | Amount | Notes |
|------|--------|-------|
| LLM-assisted development compute (rule implementation, audit-report ingestion, CI debugging) | $4,000 | Primary cost driver for solo builder |
| VM / compute infrastructure (scanning real protocols, CI, docs site) | $500 | DigitalOcean / Fly.io / GitHub Pages tier |
| Bug-bounty for external contributors (3 × $300 for high-quality rule PRs from outsiders) | $900 | Attracts third-party eyes early |
| Buffer for audit-report triage time + self-compensation for ~60h builder time | $4,600 | ~$75/h effective for 60h spread over 10 weeks |
| **Total** | **$10,000** | |

Funds are **not** for token speculation or non-project use. Accounting will be
public in a tranche log on the repo.

## 5. Timeline

- **Week 0** (grant accepted): tranche 0 paid; public kickoff post.
- **Weeks 1–3**: +10 rules → tranche 1.
- **Weeks 4–6**: +10 rules + catalogue site → tranche 2.
- **Weeks 7–8**: +5 rules + SARIF hardening + Action v1.0 → tranche 3.
- **Weeks 9–10**: 3 case studies + upstream PRs → tranche 4.
- **Week 11**: final writeup → tranche 5 (grant report).

Total: ~11 weeks. Soft buffer: 2 weeks.

## 6. Team

Solo builder. One person.

Expertise:

- Rust + AST analysis (`syn`-based visitor pattern, `proc_macro2`, SARIF
  serialization).
- Soroban-specific threat modelling: CAP-0066 archival semantics, Soroban host
  auth framework, Soroban storage model (instance / persistent / temporary TTL
  buckets).
- Reading / porting public audit reports from Halborn, Certora, WatchPug,
  Zellic V12, OtterSec into automated rule form.

No prior company, no VC funding, no legal entity. This is a self-funded side
project where the grant is the first outside capital. All work is public-facing
on GitHub: commit log, releases, CI runs, case-study scans.

Contact & code history: <https://github.com/alexandrbeher89-del/soroban-scan>.
Email: alexandrbeher89@gmail.com.

## 7. Traction

Honest status as of 2026-04-23:

- 6 releases in CHANGELOG (v0.1.0 → v0.4.0).
- 16 rules in production, each with test fixture + documented audit origin.
- 4 reproducible case-study scans on public Soroban protocols (Blend 49,
  Soroswap 39, DeFindex 99, Phoenix 129 findings).
- CAP-0066 severity regrading published (SS001 High→Low, SS006 Low→Info) —
  demonstrates calibration discipline, not ambiguous "High everywhere"
  slop.
- SARIF 2.1.0 output + reusable GitHub Action.
- x402-paywalled HTTP API: any caller can pay USDC on Base (testnet today,
  mainnet behind one env var) and get a scan back over plain HTTP. Running
  on Fly.io.
- MIT license, green CI on every commit.
- Zero external stars, zero external PRs, zero production users **as of this writing**.
  The repo is days old. Distribution phase starts post-grant.

## 8. Stellar integration

Already integrated:

- Consumes Soroban contracts (Rust crates targeting `soroban-sdk`).
- Models Soroban-specific idioms: `env.storage().persistent()/temporary()/instance()`,
  `require_auth`, `invoke_contract`, `extend_ttl`, CAP-0066 archival semantics.
- Produces SARIF 2.1.0 → GitHub Security tab ingests natively.
- Composite GitHub Action at repo root: any Soroban team can opt in with 3
  lines of YAML.

Planned integration (grant-funded):

1. **Rule coverage** for new SDK versions and protocol-level updates: cross-contract
   reentrancy patterns, Auction storage shapes, Address authorization patterns,
   TokenInterface deviations, FeeBump / SAC interactions.
2. **Distribution** through the Stellar developer ecosystem: submit to
   `awesome-soroban`, Stellar Developer Docs, SDF Discord #dev-general, direct
   maintainer outreach to Blend / Soroswap / DeFindex / Phoenix / Aqua /
   FxDAO.
3. **Public findings dashboard** tied to the case-study scans — a rolling view
   of "here is what ships with known bug classes" to drive adoption pressure.

## 9. Why Stellar / SCF-alignment

- Targets only Soroban — not multi-chain. This is intentional. Every rule is
  tuned to Soroban's host, storage model, auth framework, and TTL semantics.
  A generic Rust linter (`cargo-geiger`, `clippy`) does not and cannot replace
  this.
- Lowers the unit cost of shipping secure Soroban protocols → more builders can
  ship safely → ecosystem health.
- MIT-licensed, single-binary, no paid tier. Closes the gap that exists in the
  ecosystem today without capturing rent.

## 10. Payout details

SCF pays in **XLM to a Stellar account (G…)**. The project Stellar payout address:

```
GBMNXH6ZIOQ7L6FGI5MMXXM3QBL646HV57JGD5LRJZ3JFF6EI4PRHGHG
```

This account is freshly generated, un-funded, and held only for SCF disbursement.
The EVM address in the repo README (`0x04dd1AcaC0a5C498A8f26fcc745dc573B4EDcBFa`)
is a **tip jar**, not a grant payout address, and is out-of-band from this
submission.

## 11. Risks

| Risk | Mitigation |
|------|-----------|
| Heuristic rules fire on legitimate code (false positives) | Per-rule `--skip`, severity tiering (High / Medium / Low / Info), explicit "triage-expected" framing in README |
| Soroban SDK API changes break textual heuristics | Rules use syntactic AST matching via `syn`, not regex; rule-level tests pin against SDK version of originating audit report |
| Overlap with future commercial tools (Certora, RV, Zellic V12 in-house) | soroban-scan is explicitly positioned as the **free, fast, first-pass** tier. Complementary to semantic analysis, not a substitute |
| Solo builder bus factor | All code is MIT on GitHub; rule catalogue + contribution guide + issue templates exist so external maintainers can take over if needed |
| Adoption doesn't materialize | Upstream PRs gated on maintainer consent (tranche 4) create direct adoption surface; dashboard + public case studies create adoption pressure |

## 12. Referral

None. No prior contact with SDF or verified SCF reviewers. Cold submission on
merit of the work. (Interest Form question answered "No".)

## 13. Links

- Repo: <https://github.com/alexandrbeher89-del/soroban-scan>
- Releases: <https://github.com/alexandrbeher89-del/soroban-scan/releases>
- CI: <https://github.com/alexandrbeher89-del/soroban-scan/actions>
- Case studies: [docs/CASE-STUDIES.md](./CASE-STUDIES.md)
- CAP-0066 regrading: [docs/BLOG-CAP0066-REGRADE.md](./BLOG-CAP0066-REGRADE.md)
- First-look blog draft: [docs/BLOG-FIRST-LOOK.md](./BLOG-FIRST-LOOK.md)
