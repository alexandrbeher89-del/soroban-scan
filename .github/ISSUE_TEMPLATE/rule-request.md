---
name: New rule request
about: Propose a new detection rule based on a real audit finding
title: "[Rule request] <one-line summary>"
labels: rule-request
---

**Source audit (required):** link to the public audit report / C4/Sherlock
finding / commit that motivates this rule. Rules in this project are strictly
grounded in real audit findings — please link one.

**Bug class:** what is the general pattern? (example: "oracle price used
without staleness check")

**Minimal positive fixture (should fire):**

```rust
// paste Soroban code that embodies the bug
```

**Minimal negative fixture (should NOT fire):**

```rust
// paste similar-looking but safe code
```

**Proposed severity:** High / Medium / Low / Info.

**Proposed rule ID:** next SS0XX (we reserve IDs as PRs land).

---

If you'd like to implement it: the walkthrough is in `CONTRIBUTING.md`.
