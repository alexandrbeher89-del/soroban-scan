---
name: False positive
about: A rule fired on code that is genuinely safe
title: "[FP] SS0XX fires on <pattern>"
labels: false-positive
---

**Rule:** SS0XX

**Minimal fixture (Rust):**

```rust
// paste the smallest snippet that reproduces the false positive
```

**Why this is actually safe:**

Short explanation — e.g. "the Address is used as a factory reference, not a
user identity, because X".

**Version:** `soroban-scan --version`

---

If you'd like to submit a fix: add this fixture to `tests/fixtures/` and a
regression test to `tests/rules.rs` that asserts the rule does **not** fire.
