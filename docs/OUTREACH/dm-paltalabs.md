# DM draft — PaltaLabs / DeFindex / Soroswap maintainers

*Target:* PaltaLabs team (they maintain DeFindex + Soroswap) — Discord via
Soroswap community / Twitter DM to @paltalabs / @soroswap.
*Tone:* Peer, not pitchy. No ask.

---

**Short version (Twitter DM, < 280 chars):**

Hey — open-sourced a Soroban static analyzer (soroban-scan, MIT). Ran
v0.1.2 on DeFindex (99 findings) and Soroswap-core (39). Reproducible
reports in the repo; might save a triage hour or surface nothing. Repo:
https://github.com/alexandrbeher89-del/soroban-scan

---

**Medium version (Discord):**

Hey PaltaLabs — quick share.

I shipped a small open-source static analyzer for Soroban
([soroban-scan](https://github.com/alexandrbeher89-del/soroban-scan), MIT)
and ran v0.1.2 on your two main public repos:

- **defindex**: 99 findings across 138 files in ~0.16s
  - SS002 missing require_auth: 39 (already partly de-noised in v0.1.2)
  - SS005 integer narrowing: 31
  - SS006 persistent write without extend_ttl: 12
- **soroswap-core**: 39 findings across 74 files in ~0.07s
  - SS002: 26
  - SS012 panic/unwrap in contractimpl: 7

Full reproducible report:
https://github.com/alexandrbeher89-del/soroban-scan/blob/devin/init-soroban-scan/docs/CASE-STUDIES.md

No ask. Most findings are "triage me" not "this is broken" — the tool is
a first pass. If any of the SS002 hits on DeFindex strategies are
clearly asset-addresses the heuristic missed, a one-line fixture in the
FP-issue template makes them a regression test and they never come back.

If at any point you'd consider a CI PR (`soroban-scan . --fail-on high`),
I'm happy to draft one.

Cheers.
