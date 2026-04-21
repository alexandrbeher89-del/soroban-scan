//! SS001 — `storage().persistent().get(...).unwrap_or(...)` collapses
//! archived/expired entries into a valid zero/default, which is an
//! authorization/consistency bug observed in real Soroban audits.
//!
//! Example bad:
//! ```ignore
//! env.storage().persistent().get(&key).unwrap_or(Vec::new(env))
//! ```
//!
//! Preferred:
//! ```ignore
//! env.storage().persistent().get(&key).expect("entry must exist")
//! // or: .ok_or(Error::NotInitialized)?
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
        "persistent().get().unwrap_or(<default>) silently treats archived or expired entries as valid empty state"
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
                        description: "persistent().get().unwrap_or(...) collapses expired entries into default state",
                        severity: Severity::High,
                        file: self.path.to_path_buf(),
                        line,
                        column,
                        snippet,
                        note: Some(
                            "Use .expect()/.ok_or(...)? or explicitly handle the None case; \
                             otherwise a missing (e.g. archived-then-restored-as-default, or \
                             never-initialized) entry is indistinguishable from a legitimate \
                             empty value."
                                .into(),
                        ),
                    });
                }
            }
        }
        syn::visit::visit_expr(self, expr);
    }
}
