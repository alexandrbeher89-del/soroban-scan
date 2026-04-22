//! SS016 — a `#[contractimpl]` function named `initialize` / `init` / `setup`
//! writes privileged state without calling `.require_auth()` anywhere in its
//! body, and is not a Soroban-21 atomic constructor (`__constructor`).
//!
//! Why this matters: Soroswap, Blend and other Soroban protocols ship their
//! factories/pools with a two-step deploy procedure — `soroban contract
//! deploy` in one transaction, then `invoke initialize(...)` in a second
//! transaction. Between those two transactions anyone who watches the network
//! can front-run `initialize` and install themselves as admin / fee_to_setter,
//! or swap out the `pair_wasm_hash` the factory uses when later deploying
//! pairs. The contract code is what lets this happen: `initialize` takes a
//! privileged Address argument but never asks it to authorise the call.
//!
//! Remediation:
//!   * Soroban SDK ≥ 21: use `pub fn __constructor(...)` which runs atomically
//!     with `deploy`.
//!   * Otherwise: call `admin.require_auth()` (or
//!     `setter.require_auth()` / whatever the privileged Address is) on the
//!     first line of `initialize`. The contract stays idempotent via the
//!     existing `has_admin` / `has_total_pairs` guard.
//!
//! Severity: Medium. The vulnerability window closes the moment the legitimate
//! `initialize` succeeds, so it cannot be exploited against a running
//! deployment; but every fresh deploy (mainnet, testnet, fork, local) goes
//! through the window.

use super::helpers::{address_param_names, for_each_contractimpl_fn};
use super::Rule;
use crate::finding::{Finding, Severity};
use crate::walker::{snippet_for, span_loc};
use std::path::Path;
use syn::visit::Visit;
use syn::{ExprMethodCall, File};

pub struct Rule016;

impl Rule for Rule016 {
    fn id(&self) -> &'static str { "SS016" }
    fn name(&self) -> &'static str { "unauthenticated_initializer" }
    fn description(&self) -> &'static str {
        "contract initializer writes privileged state without require_auth — front-runnable during deploy"
    }

    fn run(&self, path: &Path, file: &File, _src: &str, out: &mut Vec<Finding>) {
        for_each_contractimpl_fn(file, |_imp, fun| {
            let name = fun.sig.ident.to_string();

            // Soroban-21 atomic constructor is safe by construction.
            if name == "__constructor" {
                return;
            }

            // Only flag functions whose name strongly suggests a one-shot
            // initialiser. We deliberately do not match `init_*` / `*_init` —
            // those patterns are often per-user bookkeeping (`init_user`, etc.)
            // rather than contract-wide init.
            let is_init = matches!(name.as_str(), "initialize" | "init" | "setup" | "__init");
            if !is_init {
                return;
            }

            // A privileged initialiser takes at least one Address parameter
            // (admin / setter / owner) or is of the form `init(cfg: SomeMeta)`
            // where the meta struct holds the admin. We use the weaker heuristic
            // "initialiser takes at least one parameter beyond `env`" — zero-arg
            // `initialize()` variants generally do nothing privileged.
            let non_env_params = fun
                .sig
                .inputs
                .iter()
                .filter(|inp| match inp {
                    syn::FnArg::Typed(pat) => {
                        if let syn::Pat::Ident(pi) = &*pat.pat {
                            pi.ident != "e" && pi.ident != "env" && pi.ident != "_e" && pi.ident != "_env"
                        } else {
                            true
                        }
                    }
                    syn::FnArg::Receiver(_) => false,
                })
                .count();
            if non_env_params == 0 {
                return;
            }

            // Scan the body for ANY `.require_auth()` / `.require_auth_for_args(...)`
            // call. Any such call at any depth is enough to clear the function —
            // this rule is about *missing* auth, not about auth on the right
            // address.
            let mut finder = AuthFinder { found: false };
            finder.visit_block(&fun.block);
            if finder.found {
                return;
            }

            let (line, column) = span_loc(fun.sig.ident.span());
            let snippet = snippet_for(path, line);

            // Pick the most plausible "admin" param for the message.
            let addr_names = address_param_names(&fun.sig);
            let subject = addr_names
                .first()
                .cloned()
                .unwrap_or_else(|| "privileged address".into());

            out.push(Finding {
                id: "SS016",
                name: "unauthenticated_initializer",
                description:
                    "initializer writes privileged state but never calls require_auth — front-runnable between deploy and initialize",
                severity: Severity::Medium,
                file: path.to_path_buf(),
                line,
                column,
                snippet,
                note: Some(format!(
                    "`{name}` does not call `{subject}.require_auth()` (or any other \
                     `.require_auth()`). On a two-step `deploy` → `initialize` deployment \
                     flow, any observer can front-run this call and take over the contract. \
                     Fixes: (a) on SDK ≥ 21, rename to `pub fn __constructor(...)` so \
                     initialisation runs atomically with deploy; or (b) add \
                     `{subject}.require_auth();` as the first line of the body — the \
                     existing `has_*` idempotency guard keeps the function one-shot."
                )),
            });
        });
    }
}

#[derive(Default)]
struct AuthFinder {
    found: bool,
}

impl<'ast> Visit<'ast> for AuthFinder {
    fn visit_expr_method_call(&mut self, mc: &'ast ExprMethodCall) {
        let m = mc.method.to_string();
        if m == "require_auth" || m == "require_auth_for_args" {
            self.found = true;
            return;
        }
        syn::visit::visit_expr_method_call(self, mc);
    }
}
