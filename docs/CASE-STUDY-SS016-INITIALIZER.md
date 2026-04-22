# Case study: SS016 — the unauthenticated-initializer anti-pattern

Reproducible findings: 2026-04-21, soroban-scan `devin/1776855155-ss016-unauth-initializer`.

## TL;DR

Most Soroban contracts ship with a two-transaction deploy flow:

```
$ soroban contract deploy --wasm factory.wasm           # tx 1 — produces CONTRACT_ID
$ soroban contract invoke --id $CONTRACT_ID -- initialize --setter $ME ...   # tx 2
```

If the contract's `initialize` function does not call `.require_auth()`,
**anyone watching the network between tx 1 and tx 2 can race tx 2 and
initialize the contract with their own admin / pair_wasm_hash / factory
address**. SS016 flags this pattern across `#[contractimpl]` functions
named `initialize` / `init` / `setup` whose body has no
`.require_auth()` and that are not Soroban-21 `__constructor` functions.

## Why this is a real class, not a stylistic nit

Soroswap's `factory.initialize` stores:

- `fee_to_setter` — the address that can change the protocol fee recipient.
- `fee_to` — the address that receives protocol fees.
- `pair_wasm_hash` — the WASM hash used for every pair `create_pair` later
  deploys.

An attacker who wins the race writes their own values. The ownership
takeover is full:

- They permanently become `fee_to_setter`, so they can rotate `fee_to` to
  themselves whenever fees are turned on.
- They can set `pair_wasm_hash` to a malicious pair contract. Every pair
  `factory.create_pair(A, B)` subsequently deploys runs attacker code and
  can drain deposited liquidity.

The vulnerability window closes the moment the legitimate `initialize`
succeeds. So for `already initialized` contracts (Soroswap and Blend
mainnet) there is no current exploit — but every fresh deployment
(testnet, fork, local integration test, any third-party redeploy of the
same WASM) goes through the window.

## Protocols scanned

| Protocol  | Initializer style   | Hits | Front-run safe? |
|-----------|---------------------|-----:|-----------------|
| Soroswap  | `pub fn initialize` |    4 | No              |
| Blend     | `pub fn initialize` |    4 | No              |
| Phoenix   | `pub fn __constructor` | 0 | Yes             |
| DeFindex  | `pub fn __constructor` | 0 | Yes             |

Command used for each repo:

```
soroban-scan <repo>/contracts | grep SS016
```

### Soroswap hits

```
MED  [SS016] contracts/factory/src/lib.rs:178:4
MED  [SS016] contracts/pair/src/lib.rs:99:8
MED  [SS016] contracts/router/src/lib.rs:390:8
MED  [SS016] contracts/token/src/contract.rs:26:12
```

- `factory.initialize(e, setter: Address, pair_wasm_hash: BytesN<32>)` —
  the main concern. No `setter.require_auth()`. The project's own deploy
  script (`scripts/old_bash/deploy_initialize_factory.sh`, lines 87–107)
  does `soroban contract deploy` and `soroban contract invoke initialize`
  as two separate shell commands, confirming the window exists.
- `router.initialize(e, factory: Address)` — same class, attacker can
  install a malicious factory.
- `pair.initialize(...)` — flagged conservatively. In practice this is
  called atomically from `factory.create_pair` inside a single host
  invocation, so the window only exists if a pair is ever deployed
  directly (not through the factory). Worth leaving the finding visible
  because forks sometimes do that.
- `token.initialize(...)` — the reference SEP-41 token template. Same
  story: deploy + initialize in two transactions on mainnet-style setups.

### Blend hits

```
MED  [SS016] backstop/src/contract.rs:180:8
MED  [SS016] emitter/src/contract.rs:77:8
MED  [SS016] pool-factory/src/pool_factory.rs:53:8
MED  [SS016] mocks/mock-pool-factory/src/pool_factory.rs:56:8
```

- `pool_factory.initialize(e, pool_init_meta: PoolInitMeta)` — the
  `PoolInitMeta` struct contains privileged state (backstop, blnd_id,
  etc.), so a front-runner can install a malicious backstop and siphon
  every pool that later gets deployed by this factory.
- `emitter.initialize(e, blnd_token, backstop, backstop_token)` — all
  three arguments are privileged contract addresses; no `require_auth`.
- `backstop.initialize(...)` — same pattern.

### Phoenix / DeFindex clean

Phoenix and DeFindex adopted Soroban-21's `pub fn __constructor(...)`
feature. A `__constructor` runs in the same host-op as `register_contract`
(or `soroban contract deploy --constructor-args`), so there is no gap
for a front-runner to slip into. SS016 explicitly skips these.

## The two-line fix

Option A — SDK ≥ 21 (preferred):

```rust
#[contractimpl]
impl Factory {
    pub fn __constructor(e: Env, setter: Address, pair_wasm_hash: BytesN<32>) {
        put_fee_to_setter(&e, &setter);
        put_fee_to(&e, setter.clone());
        put_pair_wasm_hash(&e, pair_wasm_hash);
        put_total_pairs(&e, 0);
    }
}
```

Option B — keep the old signature, close the hole:

```rust
fn initialize(e: Env, setter: Address, pair_wasm_hash: BytesN<32>) -> Result<(), FactoryError> {
    setter.require_auth();                                 // <-- only new line
    if has_total_pairs(&e) {
        return Err(FactoryError::InitializeAlreadyInitialized);
    }
    ...
}
```

The `has_total_pairs` idempotency guard is unchanged — `require_auth` does
not break one-shot semantics. It only forces the caller to be `setter`,
which is exactly what the legitimate deployer already is.

## Why this isn't caught by "just add auth everywhere"

SS002 (`missing_require_auth`) already fires on Soroswap's `initialize`
functions because they take `Address` parameters. It does **not** fire on
Blend's `pool_factory.initialize(pool_init_meta: PoolInitMeta)` because
the privileged address is inside a struct — SS002's heuristic only looks
at direct `Address` parameters. SS016 matches on function name instead,
so it catches both shapes and gives a deploy-specific remediation hint
(`__constructor`), which SS002 does not.

## Responsible disclosure status

- **Soroswap / Blend mainnet**: already initialized correctly. No current
  exploit. Both teams likely know — it is a well-understood anti-pattern
  in the wider Stellar dev community. This case study is documentation
  and tooling, not a vulnerability report.
- **Third-party forks / fresh deploys**: affected; SS016 in CI will catch
  them before deploy.
