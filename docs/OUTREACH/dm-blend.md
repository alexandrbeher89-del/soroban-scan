# DM draft — Blend Capital maintainers

*Target:* Blend core team (Discord `@mootz12` / via Blend community Discord /
Twitter DM to @blend_capital).
*Tone:* Peer, not pitchy. No ask.

---

**Short version (Twitter DM, < 280 chars):**

Hey — built a small open-source static analyzer for Soroban, soroban-scan
(MIT). Ran v0.1.2 on blend-contracts as a sanity check: 49 findings,
mostly triage-me. Thought you might find the output interesting regardless
of whether you adopt it. Repo:
https://github.com/alexandrbeher89-del/soroban-scan

---

**Medium version (Discord):**

Hey! Quick ping — I shipped an open-source Soroban static analyzer
([soroban-scan](https://github.com/alexandrbeher89-del/soroban-scan), MIT)
and ran v0.1.2 on the public `blend-contracts` repo.

Headline numbers: 49 findings across 80 files in ~0.13s. Mostly low/medium
patterns — most of the High-severity `SS002` (missing `require_auth`) hits
are actually asset-address parameters used as `TokenClient::new` args,
which v0.1.2 now partly skips via a taint heuristic. Triage is expected.

Full reproducible report:
https://github.com/alexandrbeher89-del/soroban-scan/blob/devin/init-soroban-scan/docs/CASE-STUDIES.md

No ask — if anything jumps out as a false positive, there's a dedicated
issue template and each FP becomes a regression fixture. If you're ever
open to a CI-integration PR (`--fail-on high` is three lines in a GitHub
Actions workflow), happy to put one up.

Cheers.
