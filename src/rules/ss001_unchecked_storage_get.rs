//! SS001 — `storage().persistent().get(...).unwrap_or(...)` collapses a
//! missing (never-initialized) entry into a caller-chosen default, making
//! "never written" indistinguishable from a legitimate written value.
//!
//! ### What this rule is NOT about (post CAP-0066 / Protocol 23)
//!
//! Before reading Protocol 23, a common mis-reading of this pattern was:
//! "an attacker could let the entry's TTL expire and then replay against
//! the default value". That is **wrong** on Soroban mainnet. CAP-0066
//! auto-restores archived persistent entries (or the host refuses to run
//! the invocation), so contract code **cannot** observe `unwrap_or`'s
//! default for a key that was previously written. See
//! <https://github.com/stellar/stellar-protocol/blob/master/core/cap-0066.md>.
//!
//! ### What this rule IS about
//!
//! Initialization / presence checks where the author really does mean
//! "treat missing as default". Two legitimate concerns remain:
//!
//! 1. Bugs in initialization flows (`unwrap_or(false)` on an
//!    `Initialized` flag lets anyone re-initialize).
//! 2. Code-quality smell: a silent default hides the "never set" case from
//!    readers. Prefer `.ok_or(Error::NotInitialized)?` or `.expect(...)`.
//!
//! Example (legitimate concern):
//! ```ignore
//! let initialized: bool = env.storage().instance().get(&KEY).unwrap_or(false);
//! if !initialized { /* attacker can re-initialize if admin forgot setter */ }
//! ```
use super::helpers::{chain_contains, method_chain};
use super::Rule;
use crate::finding::{Finding, Severity};
use crate::walker::{snippet_for, span_loc};
use std::path::Path;
use syn::{visit::Visit, Expr, File};

pub struct Rule001;

impl Rule for Rule001 {
    fn id(&self) -> &'static str { "SS001" }
    fn name(&self) -> &'static str { "unchecked_storage_get_with_default" }
    fn description(&self) -> &'static str {
        "persistent/instance get().unwrap_or(<default>) conflates never-initialized entries with a legitimate default value"
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

impl<'ast, 'a> Visit<'ast> for V<'a> {
    fn visit_expr(&mut self, expr: &'ast Expr) {
        if let Expr::MethodCall(mc) = expr {
            if (mc.method == "unwrap_or" || mc.method == "unwrap_or_default" || mc.method == "unwrap_or_else")
                && chain_contains(&mc.receiver, "get")
            {
                let (_, names) = method_chain(&mc.receiver);
                // Heuristic: only fire if the chain references a storage domain
                // we care about. `temporary()` reads don't have the same
                // archiving pitfalls, so we narrow to `persistent()` /
                // `instance()`.
                let is_storage = names.iter().any(|n| n == "persistent" || n == "instance");
                if is_storage {
                    let (line, column) = span_loc(mc.method.span());
                    let snippet = snippet_for(self.path, line);
                    self.out.push(Finding {
                        id: "SS001",
                        name: "unchecked_storage_get_with_default",
                        description: "persistent/instance get().unwrap_or(...) conflates never-initialized with a legitimate default",
                        severity: Severity::Low,
                        file: self.path.to_path_buf(),
                        line,
                        column,
                        snippet,
                        note: Some(
                            "Post CAP-0066 (Protocol 23) archived entries are auto-restored, so \
                             this is NOT a replay/archival vector. The real concern is \
                             initialization checks: `.unwrap_or(false)` on an `Initialized` flag \
                             lets anyone re-initialize. Prefer `.ok_or(Error::NotInitialized)?` \
                             or `.expect(...)` so \"never written\" is explicit in the control flow."
                                .into(),
                        ),
                    });
                }
            }
        }
        syn::visit::visit_expr(self, expr);
    }
}
