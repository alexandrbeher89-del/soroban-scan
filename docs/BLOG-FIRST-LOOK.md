# A first look at four public Soroban codebases through soroban-scan

*Draft blog post. Target platforms: dev.to, Medium, Stellar Discord #dev-general.*

Soroban is now three years old as a production smart-contract platform.
Dozens of protocols have shipped. Every serious DeFi contract on Stellar today
— Blend, Soroswap, DeFindex, Phoenix, Aqua — is ~30k lines of Rust, and each
of them has been audited at least once.

And yet: there is **no widely-used static analyzer for Soroban**.

In the Solidity world this gap was filled years ago by Slither, semgrep
rules, Mythril, and a dozen smaller tools. Every respectable audit engagement
starts with `slither .`, looks at the output, and only *then* spends human
hours on the residual risk. The Soroban world still starts from zero. Every
audit team builds their own mental checklist and runs it by hand.

So I built `soroban-scan` — an open-source, audit-oriented static analyzer
for Soroban contracts written in Rust. [Repo
here.](https://github.com/alexandrbeher89-del/soroban-scan)

The principle is simple: each rule is grounded in a real audit finding.
Nothing is invented. If it hasn't been paid for in a bounty, I don't codify
it. V0.1 ships 12 rules (SS001–SS012) based on patterns that Halborn,
WatchPug, and Zellic V12 have flagged in Code4rena K2 and in the Aave-port
family.

## What the tool does

`soroban-scan` is a single-binary Rust CLI. You point it at a directory of
`.rs` files, it parses them through `syn`, runs a set of AST-matching rules,
and emits findings with file, line, snippet, severity, and a remediation
hint.

```bash
$ cargo install --git https://github.com/alexandrbeher89-del/soroban-scan
$ soroban-scan path/to/contracts
```

For CI use, `--fail-on high` exits non-zero if any High-severity pattern is
detected. For triage, `--format json` emits JSON for downstream tooling.

## What four real codebases look like

I pointed the v0.1.2 binary at four public Soroban protocols. Clone, scan,
done — seconds of wall-clock each. These are not audit reports; they are
triage exercises. Every number is reproducible from the repo.

| Protocol          | Rust LoC | Files | Scan time | Findings |
| ----------------- | -------- | ----- | --------- | -------- |
| Blend protocol    |  29.7k   |   80  |  0.13 s   |   49     |
| Soroswap core     |  13.9k   |   74  |  0.07 s   |   39     |
| DeFindex          |  32.0k   |  138  |  0.16 s   |   99     |
| Phoenix protocol  |  33.0k   |   56  |  0.06 s   |  129     |

Sub-200-ms scans on 30k-line codebases. That's the regime where you can
leave it on by default in CI.

Of the findings reported, the dominant patterns (by rule ID, across all four):

- **SS005 (integer narrowing)** — `u128 as u64` and similar casts in rate /
  oracle / share math. Rarely exploitable but rarely harmless. Three of the
  four protocols ship ~20+ such casts.
- **SS002 (missing `require_auth`)** — flags `#[contractimpl]` methods that
  take an `Address` parameter but never call `require_auth()` on it. The
  single most common Soroban auth-bypass pattern. Known false-positive
  source: `Address` in Soroban is polymorphic (user IDs and contract/asset
  refs share the type), so v0.1.2 added a taint heuristic that skips Address
  params used as `Client::new(env, &x)` arguments. Triage still expected.
- **SS001 (`.unwrap_or(default)` on persistent storage)** — a pattern that
  collapses the "never set" and "just created" cases into the same default.
  For boolean flags (`unwrap_or(false)`), this often mirrors the intent.
  For numeric state (`unwrap_or(0)`), it can hide protocol invariants.
- **SS006 (persistent writes without `extend_ttl`)** — Soroban's TTL model
  auto-archives persistent entries. Writes without matching TTL extension
  are fine only when the caller always extends the TTL externally, which is
  not always the case.

Phoenix has the highest finding density (~4 per kLoC), which is expected:
the codebase is large, relatively young, and has fewer public audit rounds
than Blend. **That does not mean Phoenix is "insecure".** It means it has
more patterns that match the heuristics, which is exactly the kind of
starting point a human auditor wants.

## Why not just use Slither / semgrep?

Slither is Solidity-only and architecturally tied to the EVM IR. Porting it
to Rust+Soroban is a multi-year effort. `semgrep` can match textually on
Rust, but Soroban-specific concepts — persistent vs temporary storage, TTL
extension, `require_auth`, `contracttype`, `invoke_contract` — are not
expressible as textual patterns. They need AST context.

A focused Rust analyzer using the `syn` crate fits the job in ~1000 lines
and runs 30× faster than a semgrep pass.

## What's next

The v0.2 roadmap, based on patterns I observed in the case studies:

- **More rules.** Specifically: missing staleness checks on oracle prices;
  `saturating_sub` on balance fields (Aave-class rounding-to-zero bugs);
  ignored `deadline` parameters; partial coverage of the `TokenInterface`
  trait; and a few more.
- **Better taint analysis for SS002.** The current heuristic catches
  `Client::new(env, &x)`; adding `soroban_token_sdk` patterns and
  `try_into`/`Client::try_new` would reduce false positives further.
- **SARIF output.** GitHub Code Scanning supports it directly; this turns
  soroban-scan into a one-line drop-in for any Soroban repo's Actions
  workflow.
- **Case studies for more protocols** as they go open-source.

## Running it on your own protocol

Three lines:

```bash
cargo install --git https://github.com/alexandrbeher89-del/soroban-scan
soroban-scan ./contracts
soroban-scan ./contracts --fail-on high   # for CI
```

If it finds something, tell me. If it finds a *false positive*, **tell me
more loudly** — every FP is an issue on the repo with a minimal fixture, and
rules get tuned from there.

If you find the tool useful and would like to support further work on it,
the project accepts tips at the EVM address published in the README.

---

*soroban-scan is MIT-licensed, runs on stable Rust 1.75+, and has CI-green
builds on all pushes. PRs welcome.*
