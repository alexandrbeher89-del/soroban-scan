# DM draft — Phoenix Protocol maintainers

*Target:* Phoenix team — Discord `@joha.dev` / Phoenix community Discord /
Twitter DM to @phoenix_defi.
*Tone:* Peer, not pitchy. No ask.

---

**Short version (Twitter DM, < 280 chars):**

Hey — built an open-source Soroban static analyzer (soroban-scan, MIT).
Ran v0.1.2 on phoenix-contracts: 129 findings, ~0.06s. Mostly triage-me
patterns, but a handful worth a look (SS005 integer narrowing in rate
math). Repo: https://github.com/alexandrbeher89-del/soroban-scan

---

**Medium version (Discord):**

Hey! Ping re: Phoenix codebase.

I shipped a small open-source static analyzer for Soroban
([soroban-scan](https://github.com/alexandrbeher89-del/soroban-scan), MIT)
and ran v0.1.2 against the public `phoenix-contracts` repo.

129 findings across 56 files in ~0.06s. The notable clusters:

- **SS005 (integer narrowing)**: 52 hits — worth a careful pass, most are
  likely fine but rate-math casts can bite.
- **SS001 (.unwrap_or on persistent storage)**: 27 hits — same story,
  usually intentional but sometimes hides archival edge cases.
- **SS002 (missing require_auth)**: 32 hits — the v0.1.2 taint heuristic
  already removed some, remainder worth a look.

Full reproducible report:
https://github.com/alexandrbeher89-del/soroban-scan/blob/devin/init-soroban-scan/docs/CASE-STUDIES.md

No ask — sharing because it's probably the largest Soroban codebase
without a public audit report, and a second pass of heuristics is cheap.
False positives get regression-tested away via the issue template. If
anything in the report looks genuinely wrong, tell me and I'll tune the
rule.

Cheers.
