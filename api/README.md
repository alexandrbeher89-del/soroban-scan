# soroban-scan API (x402 paywall)

Pay-per-call HTTP API wrapping the [soroban-scan](../README.md) static analyzer.
Each paid request settles USDC on-chain via the [x402](https://x402.org) protocol;
payments go directly to the merchant wallet — no accounts, no API keys on the
client side.

## Live deployment

```
https://soroban-scan-api-qhldjpbq.fly.dev/
```

Status: **live on Base mainnet** via the [OpenX402](https://facilitator.openx402.ai)
facilitator — permissionless, no signup, no API keys. Paid endpoints quote real
Base USDC (`0x833589fCD6eDb6E08f4c7C32D4f71b54bdA02913`). Switch to Base-Sepolia
or any other x402 facilitator via the environment variables below.

## Endpoints

| Method | Path         | Paid  | Price  | Description                               |
|--------|--------------|-------|--------|-------------------------------------------|
| GET    | `/`          | no    | —      | landing JSON (status + endpoint index)    |
| GET    | `/rules`     | no    | —      | rule catalog (id, severity, title)        |
| GET    | `/healthz`   | no    | —      | liveness probe                            |
| POST   | `/scan`      | yes   | $0.01  | scan a single Rust source blob            |
| POST   | `/scan/repo` | yes   | $0.05  | clone a public git repo and scan it       |

## Calling a paid endpoint

Clients speak [x402](https://x402.org) natively. In Python:

```bash
pip install "x402[httpx,evm]"
```

```python
import httpx
from x402.clients.httpx import x402HttpxClient
from eth_account import Account

signer = Account.from_key("0x...")          # any EVM signer
async with x402HttpxClient(signer, "https://soroban-scan-api-qhldjpbq.fly.dev") as c:
    r = await c.post("/scan", json={"source": open("contract.rs").read()})
    print(r.json())
```

No header construction, no facilitator round-trip on the client side — the SDK
does it all. You just need a signer funded with Base mainnet USDC. For
development against a testnet deployment, pull Base-Sepolia faucet USDC from
https://faucet.circle.com/ and redeploy with `X402_NETWORK=base-sepolia`.

## Request schemas

### POST /scan

```json
{
  "source": "use soroban_sdk::...",
  "include_tests": false,
  "skip_rules": ["SS006"]
}
```

Response: `{ "digest": "...", "bytes": N, "findings": { ...soroban-scan JSON... } }`.

### POST /scan/repo

```json
{
  "git_url": "https://github.com/...",
  "ref":     "main",
  "subdir":  "contracts/lending-pool",
  "include_tests": false,
  "skip_rules": []
}
```

Response: `{ "git_url": "...", "ref": "...", "subdir": "...", "findings": {...} }`.

## Environment variables

| Var                      | Default                             | Purpose                                    |
|--------------------------|-------------------------------------|--------------------------------------------|
| `PAY_TO_ADDRESS`         | `0x04dd...cBFa`                     | Merchant wallet receiving USDC             |
| `X402_NETWORK`           | `base`                              | `base` (mainnet) or `base-sepolia` (testnet) |
| `FACILITATOR_URL`        | `https://facilitator.openx402.ai`   | Verifier + settler                         |
| `SOROBAN_SCAN_BIN`       | `./soroban-scan`                    | Path to scanner binary                     |
| `SCAN_TIMEOUT_SECS`      | `60`                                | Per-scan subprocess timeout                |
| `REPO_SCAN_TIMEOUT_SECS` | `180`                               | Per-repo scan subprocess timeout           |
| `MAX_SOURCE_BYTES`       | `524288`                            | Upper bound on `/scan` payload             |
| `REPO_CLONE_MAX_BYTES`   | `52428800`                          | Upper bound on repo clone size             |
| `X402_DISABLED`          | `0`                                 | Set to `1` for free local dev              |

## Switching facilitators / networks

The default deployment uses [OpenX402](https://facilitator.openx402.ai) on Base
mainnet. To run elsewhere, override the env vars:

1. **Base-Sepolia testnet** — `X402_NETWORK=base-sepolia`,
   `FACILITATOR_URL=https://x402.org/facilitator` (Coinbase's public testnet
   facilitator). Request faucet USDC at https://faucet.circle.com/.
2. **Coinbase CDP facilitator on mainnet** — free tier 1000 tx/month,
   requires a CDP account (5-min signup). Set
   `FACILITATOR_URL=https://api.cdp.coinbase.com/platform/v2/x402` and provide
   the CDP API key via whatever auth pattern your deployment supports.
3. **Alternative permissionless facilitators** — e.g. PayAI
   (`https://facilitator.payai.network`). Coverage and fee structure vary.
4. **Self-settlement** — fund the merchant wallet with ~$1 of ETH on Base and
   have the server call USDC's `transferWithAuthorization` directly. Removes
   the facilitator dependency but adds a gas-funded hot wallet.

## Local development

```bash
cd api/
python -m venv .venv && source .venv/bin/activate
pip install -e .
# Copy the statically linked scanner binary next to main.py:
cp ../target/x86_64-unknown-linux-musl/release/soroban-scan .
X402_DISABLED=1 uvicorn main:app --reload
```

## Source

- API: this directory
- Scanner: [../src/](../src/)
- Protocol: https://x402.org
- Deploy target: [Fly.io](https://fly.io/) (works with any container platform)
