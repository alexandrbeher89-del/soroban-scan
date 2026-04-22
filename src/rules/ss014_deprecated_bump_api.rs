//! SS014 — use of the deprecated Soroban TTL API `.bump(...)` on a storage
//! handle. The SDK renamed `bump` → `extend_ttl` around v20 (stabilized in
//! Protocol 20 preview / mainnet GA). Code still calling `.bump(...)` is
//! either compiling against an obsolete SDK or was mechanically ported
//! without adopting the new name.
//!
//! Detection: `Expr::MethodCall` with `method == "bump"` and a receiver chain
//! that goes through one of the Soroban storage-kind handles
//! (`.persistent()`, `.temporary()`, `.instance()`). This avoids flagging
//! unrelated `.bump(...)` methods in the wider Rust ecosystem (`semver::Version`,
//! `Duration`, etc.) because none of those live behind a storage handle.
//!
//! Severity: Info — this is a stylistic / forward-compatibility hint, not a
//! security issue. The `bump` alias still works on older SDKs.

use super::Rule;
use crate::finding::{Finding, Severity};
use crate::walker::{snippet_for, span_loc};
use std::path::Path;
use syn::visit::Visit;
use syn::{Expr, ExprMethodCall, File};

pub struct Rule014;

impl Rule for Rule014 {
    fn id(&self) -> &'static str { "SS014" }
    fn name(&self) -> &'static str { "deprecated_bump_api" }
    fn description(&self) -> &'static str {
        "deprecated Soroban TTL API `.bump(...)` on storage handle — use `.extend_ttl(...)`"
    }

    fn run(&self, path: &Path, file: &File, _src: &str, out: &mut Vec<Finding>) {
        let mut v = V { path, out };
        v.visit_file(file);
    }
}

struct V<'a> {
    path: &'a Path,
    out: &'a mut Vec<Finding>,
}

/// Walk the receiver chain; return `true` if any method call in the chain is
/// `persistent`, `temporary`, or `instance` (the Soroban storage-kind
/// handles). Unwraps `Paren` / `Reference` / `Cast` wrappers transparently.
fn chain_has_storage_kind(expr: &Expr) -> bool {
    match expr {
        Expr::MethodCall(mc) => {
            let m = mc.method.to_string();
            if m == "persistent" || m == "temporary" || m == "instance" {
                return true;
            }
            chain_has_storage_kind(&mc.receiver)
        }
        Expr::Paren(p) => chain_has_storage_kind(&p.expr),
        Expr::Reference(r) => chain_has_storage_kind(&r.expr),
        Expr::Cast(c) => chain_has_storage_kind(&c.expr),
        _ => false,
    }
}

impl<'ast, 'a> Visit<'ast> for V<'a> {
    fn visit_expr_method_call(&mut self, mc: &'ast ExprMethodCall) {
        if mc.method == "bump" && chain_has_storage_kind(&mc.receiver) {
            let (line, column) = span_loc(mc.method.span());
            let snippet = snippet_for(self.path, line);
            self.out.push(Finding {
                id: "SS014",
                name: "deprecated_bump_api",
                description:
                    "`.bump(...)` on a storage handle is deprecated; rename to `.extend_ttl(...)`",
                severity: Severity::Info,
                file: self.path.to_path_buf(),
                line,
                column,
                snippet,
                note: Some(
                    "The Soroban SDK renamed `bump` to `extend_ttl` around v20. \
                     Replace `.bump(&key, threshold, extend_to)` with \
                     `.extend_ttl(&key, threshold, extend_to)` for persistent/temporary \
                     and `.extend_ttl(threshold, extend_to)` for instance. The call \
                     signature is unchanged."
                        .into(),
                ),
            });
        }
        // Keep walking so nested calls (e.g. inside arguments) are also seen.
        syn::visit::visit_expr_method_call(self, mc);
    }
}
