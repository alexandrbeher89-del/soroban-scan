"""
soroban-scan HTTP API with a minimal x402 paywall.

Wraps the `soroban-scan` static analyzer binary as a pay-per-call web service.
Clients pay small USDC amounts on Base (via the x402 protocol) per request;
payments settle directly to the wallet address configured by `PAY_TO_ADDRESS`.

Free endpoints:
  GET  /                -> landing page (JSON status + docs link)
  GET  /rules           -> list rule metadata
  GET  /healthz         -> health probe

Paid endpoints:
  POST /scan            -> scan a single Rust source blob               ($0.01)
  POST /scan/repo       -> clone a public git repo and scan it          ($0.05)

This module implements the server side of x402 directly (verify/settle via a
configurable facilitator URL) instead of pulling in the heavy `cdp-sdk` chain
of deps (web3 + solders + cryptography) that the `fastapi-x402` wrapper
drags in. That keeps the container image small and `pip install` fast.

References:
  - https://x402.org
  - https://docs.cdp.coinbase.com/x402/docs/facilitator
"""

from __future__ import annotations

import base64
import hashlib
import json
import os
import shutil
import subprocess
import tempfile
import time
from pathlib import Path
from typing import Any

import httpx
from fastapi import Body, FastAPI, HTTPException, Request
from fastapi.responses import JSONResponse
from pydantic import BaseModel, Field

# ---------------------------------------------------------------------------

_DEFAULT_SCAN_BIN = str(Path(__file__).resolve().parent / "soroban-scan")
SCAN_BIN = os.environ.get("SOROBAN_SCAN_BIN", _DEFAULT_SCAN_BIN)
SCAN_TIMEOUT_SECS = int(os.environ.get("SCAN_TIMEOUT_SECS", "60"))
REPO_SCAN_TIMEOUT_SECS = int(os.environ.get("REPO_SCAN_TIMEOUT_SECS", "180"))
MAX_SOURCE_BYTES = int(os.environ.get("MAX_SOURCE_BYTES", str(512 * 1024)))
REPO_CLONE_MAX_BYTES = int(os.environ.get("REPO_CLONE_MAX_BYTES", str(50 * 1024 * 1024)))

PAY_TO_ADDRESS = os.environ.get(
    "PAY_TO_ADDRESS", "0x04dd1AcaC0a5C498A8f26fcc745dc573B4EDcBFa"
)
X402_NETWORK = os.environ.get("X402_NETWORK", "base-sepolia")
FACILITATOR_URL = os.environ.get("FACILITATOR_URL", "https://x402.org/facilitator")
X402_DISABLED = os.environ.get("X402_DISABLED", "0") == "1"  # for local smoke tests

# USDC contract addresses per network (source: https://www.circle.com/en/usdc/developer)
USDC_BY_NETWORK = {
    "base":         {"address": "0x833589fCD6eDb6E08f4c7C32D4f71b54bdA02913", "version": "2"},
    "base-sepolia": {"address": "0x036CbD53842c5426634e7929541eC2318f3dCF7e", "version": "2"},
}

# ---------------------------------------------------------------------------

app = FastAPI(
    title="soroban-scan API",
    version="0.1.0",
    description=(
        "Pay-per-call static analysis for Soroban smart contracts. "
        "Each paid request settles USDC on-chain via the x402 protocol."
    ),
)


class ScanRequest(BaseModel):
    source: str = Field(..., description="Raw Rust source text to scan.", max_length=MAX_SOURCE_BYTES)
    include_tests: bool = False
    skip_rules: list[str] = Field(default_factory=list)


class RepoScanRequest(BaseModel):
    git_url: str = Field(..., description="HTTPS URL of a public git repository.")
    ref: str | None = Field(None, description="Optional branch / tag / commit to check out.")
    subdir: str | None = Field(None, description="Optional sub-directory inside the repo.")
    include_tests: bool = False
    skip_rules: list[str] = Field(default_factory=list)


# ---------------------------------------------------------------------------

RULES: list[dict[str, str]] = [
    {"id": "SS001", "severity": "Low",    "title": "storage::get without unwrap_or"},
    {"id": "SS002", "severity": "Medium", "title": "missing require_auth on Address param"},
    {"id": "SS003", "severity": "Low",    "title": "unbounded Vec iteration"},
    {"id": "SS004", "severity": "Medium", "title": "invoke_contract reentrancy risk"},
    {"id": "SS005", "severity": "Medium", "title": "u128 -> smaller-int downcast truncation"},
    {"id": "SS006", "severity": "Info",   "title": "persistent storage without extend_ttl"},
    {"id": "SS007", "severity": "Medium", "title": "transfer after state read (CEI violation)"},
    {"id": "SS009", "severity": "Medium", "title": "floating-point arithmetic in contract code"},
    {"id": "SS012", "severity": "Low",    "title": "panic! / unwrap in contract body"},
    {"id": "SS013", "severity": "High",   "title": "ledger timestamp/sequence used as entropy"},
    {"id": "SS014", "severity": "Low",    "title": "deprecated .bump() TTL API"},
    {"id": "SS015", "severity": "Medium", "title": "floating-point type declared (f32/f64/f128)"},
    {"id": "SS016", "severity": "High",   "title": "unauthenticated initialize() (front-runnable deploy)"},
]


# ---------------------------------------------------------------------------
# x402 payment middleware
#
# Per-route price map is populated by the `@paid(price_usd)` decorator. When a
# request targets a paid route:
#   1. If no `X-PAYMENT` header -> return 402 with payment requirements JSON.
#   2. Else, POST to facilitator `/verify`. If the facilitator rejects, return 402
#      with the facilitator reason.
#   3. Otherwise execute the route handler, then POST to facilitator `/settle`
#      and attach the settlement tx info to `X-PAYMENT-RESPONSE`.
#
# Amounts are encoded in token base units: $0.01 of USDC (6 decimals) = 10000.

_PAID_ROUTES: dict[str, int] = {}


def paid(price_usd: str):
    """
    Decorator: mark a FastAPI route as x402-paid. `price_usd` is of the form
    "$0.01". The amount is converted to USDC base units (6 decimals).
    """
    assert price_usd.startswith("$"), "price must start with $"
    amount_base = int(round(float(price_usd[1:]) * 10**6))

    def _decorator(func):
        endpoint = "/" + func.__name__.replace("_", "/")
        _PAID_ROUTES[endpoint] = amount_base  # path matched in middleware
        func._x402_price = amount_base
        return func

    return _decorator


def _payment_requirements(path: str, amount_base: int) -> dict[str, Any]:
    usdc = USDC_BY_NETWORK.get(X402_NETWORK, USDC_BY_NETWORK["base-sepolia"])
    return {
        "x402Version": 1,
        "error": "X-PAYMENT header is required",
        "accepts": [
            {
                "scheme": "exact",
                "network": X402_NETWORK,
                "maxAmountRequired": str(amount_base),
                "resource": path,
                "description": f"soroban-scan API -> {path}",
                "mimeType": "application/json",
                "payTo": PAY_TO_ADDRESS,
                "maxTimeoutSeconds": 300,
                "asset": usdc["address"],
                "extra": {"name": "USDC", "version": usdc["version"]},
            }
        ],
    }


def _register_paid(route_path: str, price_usd: str) -> int:
    amount_base = int(round(float(price_usd.lstrip("$")) * 10**6))
    _PAID_ROUTES[route_path] = amount_base
    return amount_base


def _requirements_for(path: str) -> dict[str, Any] | None:
    amount = _PAID_ROUTES.get(path)
    if amount is None:
        return None
    return _payment_requirements(path, amount)


async def _facilitator_post(sub_path: str, payload: dict[str, Any]) -> dict[str, Any]:
    url = FACILITATOR_URL.rstrip("/") + sub_path
    async with httpx.AsyncClient(timeout=30.0) as client:
        r = await client.post(url, json=payload)
    if r.status_code >= 400:
        raise HTTPException(
            status_code=502,
            detail={"facilitator": sub_path, "status": r.status_code, "body": r.text[:500]},
        )
    try:
        return r.json()
    except Exception as e:
        raise HTTPException(status_code=502, detail=f"facilitator bad JSON: {e}")


@app.middleware("http")
async def x402_middleware(request: Request, call_next):
    if X402_DISABLED or request.url.path not in _PAID_ROUTES:
        return await call_next(request)

    amount_base = _PAID_ROUTES[request.url.path]
    accepts_body = _payment_requirements(request.url.path, amount_base)
    requirements = accepts_body["accepts"][0]

    x_payment = request.headers.get("x-payment") or request.headers.get("X-PAYMENT")
    if not x_payment:
        return JSONResponse(status_code=402, content=accepts_body)

    try:
        decoded = json.loads(base64.b64decode(x_payment + "==").decode("utf-8"))
    except Exception as e:
        return JSONResponse(
            status_code=402,
            content={**accepts_body, "error": f"invalid X-PAYMENT encoding: {e}"},
        )

    verify = await _facilitator_post(
        "/verify",
        {"x402Version": 1, "paymentPayload": decoded, "paymentRequirements": requirements},
    )
    if not verify.get("isValid"):
        return JSONResponse(
            status_code=402,
            content={**accepts_body, "error": "verify_failed", "facilitator": verify},
        )

    response = await call_next(request)

    # Only settle when the handler delivered the paid service (2xx). If the
    # handler returned an error (413 payload too large, 504 scanner timeout,
    # 500 scanner crash, etc.), the client should not be charged on-chain.
    if response.status_code >= 400:
        return response

    try:
        settle = await _facilitator_post(
            "/settle",
            {"x402Version": 1, "paymentPayload": decoded, "paymentRequirements": requirements},
        )
        response.headers["X-PAYMENT-RESPONSE"] = base64.b64encode(
            json.dumps(settle).encode("utf-8")
        ).decode("ascii")
    except HTTPException as e:
        # Handler already ran and succeeded; don't hide the response if the
        # settle round-trip failed, just surface a warning header the client
        # can inspect.
        response.headers["X-PAYMENT-SETTLE-ERROR"] = str(e.detail)[:200]

    return response


# Register paid routes up-front so the middleware knows amounts.
_register_paid("/scan",      "$0.01")
_register_paid("/scan/repo", "$0.05")


# ---------------------------------------------------------------------------


def _run_scanner(path: Path, include_tests: bool, skip: list[str], timeout: int) -> Any:
    args = [SCAN_BIN, "--format", "json"]
    if include_tests:
        args.append("--include-tests")
    if skip:
        args.extend(["--skip", ",".join(skip)])
    args.append(str(path))

    try:
        proc = subprocess.run(args, capture_output=True, text=True, timeout=timeout, check=False)
    except subprocess.TimeoutExpired:
        raise HTTPException(status_code=504, detail=f"scan exceeded {timeout}s timeout")
    except FileNotFoundError:
        raise HTTPException(status_code=500, detail=f"scanner binary not found at {SCAN_BIN}")

    if proc.returncode not in (0, 1):  # 1 = findings reported; 0 = no findings
        raise HTTPException(
            status_code=500,
            detail={"scanner_exit_code": proc.returncode, "stderr": proc.stderr[-2000:]},
        )

    try:
        return json.loads(proc.stdout or "{}")
    except json.JSONDecodeError as e:
        raise HTTPException(status_code=500, detail=f"scanner produced non-JSON output: {e}")


def _source_digest(source: str) -> str:
    return hashlib.sha256(source.encode("utf-8")).hexdigest()


def _clone_repo(git_url: str, ref: str | None, workdir: Path) -> Path:
    if not git_url.startswith(("https://", "git://")):
        raise HTTPException(status_code=400, detail="only https:// or git:// URLs accepted")

    target = workdir / "repo"
    clone_args = [
        "git", "clone",
        "--depth=1",
        "--single-branch",
        "--no-tags",
        "--recurse-submodules=no",
    ]
    if ref:
        clone_args.extend(["--branch", ref])
    clone_args.extend([git_url, str(target)])

    try:
        proc = subprocess.run(clone_args, capture_output=True, text=True, timeout=60, check=False)
    except subprocess.TimeoutExpired:
        raise HTTPException(status_code=504, detail="git clone exceeded 60s timeout")
    except FileNotFoundError:
        raise HTTPException(status_code=500, detail="git binary not found in container")

    if proc.returncode != 0:
        raise HTTPException(
            status_code=400,
            detail={"git_exit_code": proc.returncode, "stderr": proc.stderr[-2000:]},
        )

    total = 0
    for p in target.rglob("*"):
        if p.is_file():
            total += p.stat().st_size
            if total > REPO_CLONE_MAX_BYTES:
                shutil.rmtree(target, ignore_errors=True)
                raise HTTPException(
                    status_code=413,
                    detail=f"cloned repo exceeds {REPO_CLONE_MAX_BYTES} byte limit",
                )
    return target


# ---------------------------------------------------------------------------


@app.get("/")
def index() -> dict[str, Any]:
    return {
        "service": "soroban-scan API",
        "version": "0.1.0",
        "payment": {
            "protocol": "x402",
            "network": X402_NETWORK,
            "pay_to": PAY_TO_ADDRESS,
            "asset": "USDC",
            "facilitator": FACILITATOR_URL,
        },
        "endpoints": {
            "GET /":           {"paid": False, "description": "this page"},
            "GET /rules":      {"paid": False, "description": "rule catalog"},
            "GET /healthz":    {"paid": False, "description": "liveness probe"},
            "POST /scan":      {"paid": True, "price_usd": 0.01, "description": "scan a single Rust source blob"},
            "POST /scan/repo": {"paid": True, "price_usd": 0.05, "description": "clone a public git repo and scan it"},
        },
        "source": "https://github.com/alexandrbeher89-del/soroban-scan",
        "docs":   "/docs",
    }


@app.get("/healthz")
def healthz() -> dict[str, Any]:
    return {"ok": True, "ts": int(time.time())}


@app.get("/rules")
def list_rules() -> dict[str, Any]:
    return {"count": len(RULES), "rules": RULES}


@app.post("/scan")
def scan_source(req: ScanRequest = Body(...)) -> dict[str, Any]:
    source_bytes = req.source.encode("utf-8")
    if len(source_bytes) > MAX_SOURCE_BYTES:
        raise HTTPException(
            status_code=413,
            detail=f"source too large ({len(source_bytes)} bytes; max {MAX_SOURCE_BYTES})",
        )

    with tempfile.TemporaryDirectory(prefix="scan-src-") as td:
        path = Path(td) / "lib.rs"
        path.write_bytes(source_bytes)
        findings = _run_scanner(path, req.include_tests, req.skip_rules, SCAN_TIMEOUT_SECS)

    return {
        "digest": _source_digest(req.source),
        "bytes": len(source_bytes),
        "findings": findings,
    }


@app.post("/scan/repo")
def scan_repo(req: RepoScanRequest = Body(...)) -> dict[str, Any]:
    with tempfile.TemporaryDirectory(prefix="scan-repo-") as td:
        workdir = Path(td)
        repo = _clone_repo(req.git_url, req.ref, workdir)

        scan_path = repo
        if req.subdir:
            candidate = (repo / req.subdir).resolve()
            if not candidate.is_relative_to(repo.resolve()):
                raise HTTPException(status_code=400, detail="subdir escapes repository root")
            if not candidate.exists():
                raise HTTPException(status_code=404, detail=f"subdir not found: {req.subdir}")
            scan_path = candidate

        findings = _run_scanner(scan_path, req.include_tests, req.skip_rules, REPO_SCAN_TIMEOUT_SECS)

    return {
        "git_url": req.git_url,
        "ref": req.ref,
        "subdir": req.subdir,
        "findings": findings,
    }


@app.exception_handler(Exception)
async def on_error(_request, exc: Exception):  # type: ignore[override]
    return JSONResponse(
        status_code=500,
        content={"error": type(exc).__name__, "detail": str(exc)[:500]},
    )
