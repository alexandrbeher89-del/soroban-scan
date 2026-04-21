//! SS003 — iterating over a `Vec` or `Map` whose contents grow with
//! user/admin actions without an explicit bound. Soroban's CPU budget (100M
//! instructions per tx) can be exhausted by unbounded loops, giving an
//! attacker a cheap griefing/DoS vector.
//!
//! Heuristic: flag `for _ in <expr>.iter()` / `for _ in <expr>` where `<expr>`
//! is a local or field whose type is `Vec<_>` / `Map<_, _>` or clearly comes
//! from storage (`.get::<Vec<..>>(...)`), and the loop body does not look like
//! it contains a `break` with a counter guard.

use super::Rule;
use crate::finding::{Finding, Severity};
use crate::walker::{snippet_for, span_loc};
use std::path::Path;
use syn::visit::Visit;
use syn::{Expr, ExprForLoop, File};

pub struct Rule003;

impl Rule for Rule003 {
    fn id(&self) -> &'static str { "SS003" }
    fn name(&self) -> &'static str { "unbounded_loop_over_storage_collection" }
    fn description(&self) -> &'static str {
        "for-loop over a Vec/Map obtained from storage without an explicit length guard risks exhausting the 100M CPU budget"
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
    fn visit_expr_for_loop(&mut self, fl: &'ast ExprForLoop) {
        // Heuristic: we only fire if the iterated expression's textual form
        // looks like it came from storage OR the local name suggests a Vec
        // from storage (`reserves`, `users`, `list`, `entries`, `bids`…).
        let iter_text = quote::quote!(#fl).to_string();
        let looks_like_storage = iter_text.contains(".iter()")
            && (iter_text.contains("storage")
                || iter_text.contains("get_reserves_list")
                || iter_text.contains("_list")
                || iter_text.contains("whitelist")
                || iter_text.contains("blacklist"));

        let stmts = &fl.body.stmts;
        let body_text = quote::quote!(#(#stmts)*).to_string();
        // Skip if the body has a `break` or a `.take(N)` adapter on the iterator.
        let already_bounded = body_text.contains("break")
            || iter_text.contains(".take(")
            || iter_text.contains(".chunks(");

        if looks_like_storage && !already_bounded {
            let (line, column) = span_loc(fl.for_token.span);
            let snippet = snippet_for(self.path, line);
            self.out.push(Finding {
                id: "SS003",
                name: "unbounded_loop_over_storage_collection",
                description:
                    "for-loop over a storage-backed collection without a length guard",
                severity: Severity::Medium,
                file: self.path.to_path_buf(),
                line,
                column,
                snippet,
                note: Some(
                    "Bound the iteration (`.take(MAX)`) or enforce a constant upper bound on \
                     the collection's length at write-time. Unbounded loops become DoS once \
                     an attacker can influence the collection's size."
                        .into(),
                ),
            });
        }

        syn::visit::visit_expr_for_loop(self, fl);
    }

    fn visit_expr(&mut self, e: &'ast Expr) {
        syn::visit::visit_expr(self, e);
    }
}
