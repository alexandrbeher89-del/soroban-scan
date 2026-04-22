//! SS002 — a `#[contractimpl]` method takes an `Address` parameter but never
//! calls `.require_auth()` (or `.require_auth_for_args()`) on it. This is the
//! single most common Soroban auth-bypass pattern.
//!
//! False-positive friendly: view-only functions (`get_*`, `query_*`, starting
//! with `view_`) and functions that clearly route through another contract
//! that performs auth are skipped if their name suggests it, but triage by
//! hand is still expected.

use super::helpers::{address_param_names, for_each_contractimpl_fn, path_ends_with};
use super::Rule;
use crate::finding::{Finding, Severity};
use crate::walker::{snippet_for, span_loc};
use std::path::Path;
use syn::visit::Visit;
use syn::{Expr, File};

/// Collects identifier names that are passed as the 2nd argument to any
/// `<SomeClient>::new(env, &addr)` or `<SomeClient>::new(env, addr.clone())`
/// call — a strong signal that `addr` is an asset/contract reference rather
/// than a user identity, and therefore should not be expected to `require_auth`.
#[derive(Default)]
struct ClientArgCollector {
    contract_refs: Vec<String>,
}

impl<'ast> Visit<'ast> for ClientArgCollector {
    fn visit_expr_call(&mut self, call: &'ast syn::ExprCall) {
        if let Expr::Path(p) = &*call.func {
            let is_new = p
                .path
                .segments
                .last()
                .map(|s| s.ident == "new")
                .unwrap_or(false);
            if is_new && call.args.len() >= 2 {
                if let Some(name) = extract_ident(&call.args[1]) {
                    self.contract_refs.push(name);
                }
            }
        }
        syn::visit::visit_expr_call(self, call);
    }
}

/// Extract the top-level identifier from expressions such as `x`, `&x`,
/// `x.clone()`, `&x.clone()`, `addr.clone()`.
fn extract_ident(expr: &Expr) -> Option<String> {
    match expr {
        Expr::Path(p) => p.path.get_ident().map(|i| i.to_string()),
        Expr::Reference(r) => extract_ident(&r.expr),
        Expr::MethodCall(mc) if mc.method == "clone" => extract_ident(&mc.receiver),
        _ => None,
    }
}

pub struct Rule002;

impl Rule for Rule002 {
    fn id(&self) -> &'static str { "SS002" }
    fn name(&self) -> &'static str { "missing_require_auth" }
    fn description(&self) -> &'static str {
        "public contract method takes an Address parameter but never invokes require_auth on it"
    }

    fn run(&self, path: &Path, file: &File, _src: &str, out: &mut Vec<Finding>) {
        for_each_contractimpl_fn(file, |_imp, fun| {
            let name = fun.sig.ident.to_string();
            // Skip obviously view methods.
            if name.starts_with("get_")
                || name.starts_with("query_")
                || name.starts_with("view_")
                || name == "name"
                || name == "symbol"
                || name == "decimals"
                || name == "balance"
                || name == "balance_of"
                || name == "allowance"
                || name == "total_supply"
            {
                return;
            }

            // Skip functions marked with an attribute that starts with
            // `#[view]` (non-standard but sometimes used).
            if fun.attrs.iter().any(|a| path_ends_with(a.path(), "view")) {
                return;
            }

            let addr_params = address_param_names(&fun.sig);
            if addr_params.is_empty() {
                return;
            }

            // Walk the body and collect all `.require_auth` / `.require_auth_for_args` receivers.
            let mut finder = AuthFinder::default();
            finder.visit_block(&fun.block);
            let authed: Vec<String> = finder.authed;

            // Collect parameters that are clearly asset/contract references
            // (passed as the 2nd arg to `<Client>::new(env, &x)`).
            let mut client_args = ClientArgCollector::default();
            client_args.visit_block(&fun.block);

            // Any Address param that was never auth'd AND not obviously a
            // contract reference triggers a finding (reported once per
            // function for the first such param).
            let missing: Vec<String> = addr_params
                .iter()
                .filter(|a| !authed.iter().any(|e| e == *a))
                .filter(|a| !client_args.contract_refs.iter().any(|e| e == *a))
                .cloned()
                .collect();

            if !missing.is_empty() {
                let (line, column) = span_loc(fun.sig.ident.span());
                let snippet = snippet_for(path, line);
                out.push(Finding {
                    id: "SS002",
                    name: "missing_require_auth",
                    description:
                        "Address parameter in public contract function without matching require_auth()",
                    severity: Severity::High,
                    file: path.to_path_buf(),
                    line,
                    column,
                    snippet,
                    note: Some(format!(
                        "Parameter(s) never auth'd: {}. Add `{}.require_auth()` early in the function, or document why the caller is trusted.",
                        missing.join(", "),
                        missing[0],
                    )),
                });
            }
        });
    }
}

#[derive(Default)]
struct AuthFinder {
    authed: Vec<String>,
}

impl<'ast> Visit<'ast> for AuthFinder {
    fn visit_expr_method_call(&mut self, mc: &'ast syn::ExprMethodCall) {
        if mc.method == "require_auth" || mc.method == "require_auth_for_args" {
            if let Expr::Path(p) = &*mc.receiver {
                if let Some(seg) = p.path.segments.last() {
                    self.authed.push(seg.ident.to_string());
                }
            }
            // Also handle `&addr.require_auth()` and `addr.clone().require_auth()`.
            if let Expr::Reference(r) = &*mc.receiver {
                if let Expr::Path(p) = &*r.expr {
                    if let Some(seg) = p.path.segments.last() {
                        self.authed.push(seg.ident.to_string());
                    }
                }
            }
            if let Expr::MethodCall(inner) = &*mc.receiver {
                if let Expr::Path(p) = &*inner.receiver {
                    if let Some(seg) = p.path.segments.last() {
                        self.authed.push(seg.ident.to_string());
                    }
                }
            }
        }
        syn::visit::visit_expr_method_call(self, mc);
    }
}
