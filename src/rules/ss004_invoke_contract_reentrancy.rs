//! SS004 — detects state writes that follow a cross-contract invocation in the
//! same function without an intervening reentrancy guard. Soroban allows
//! re-entry through `env.invoke_contract(...)` and `Client::new(...).method()`
//! calls; protocols implementing CEI (Checks-Effects-Interactions) put
//! state changes BEFORE external calls.
//!
//! Heuristic: in a function body, find the last external-call expression
//! (method call whose receiver is an `env.invoke_contract` OR a generated
//! `Client::...` call), then check whether any subsequent statement writes
//! to `env.storage()`. If so, flag it.

use super::Rule;
use crate::finding::{Finding, Severity};
use crate::walker::{snippet_for, span_loc};
use std::path::Path;
use syn::visit::Visit;
use syn::{File, Stmt};

pub struct Rule004;

impl Rule for Rule004 {
    fn id(&self) -> &'static str { "SS004" }
    fn name(&self) -> &'static str { "state_write_after_external_call" }
    fn description(&self) -> &'static str {
        "storage write after an external cross-contract call in the same function (potential reentrancy / CEI violation)"
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

fn stmt_text(s: &Stmt) -> String {
    quote::quote!(#s).to_string()
}

fn is_external_call(text: &str) -> bool {
    text.contains("invoke_contract")
        || text.contains("Client::new")
        || text.contains("token::Client")
        || text.contains("TokenClient::new")
}

fn is_state_write(text: &str) -> bool {
    text.contains("storage()") && (text.contains(".set(") || text.contains(".update(") || text.contains(".remove("))
}

impl<'ast, 'a> Visit<'ast> for V<'a> {
    fn visit_item_fn(&mut self, f: &'ast syn::ItemFn) {
        self.check_block(&f.block, f.sig.ident.span());
        syn::visit::visit_item_fn(self, f);
    }

    fn visit_impl_item_fn(&mut self, f: &'ast syn::ImplItemFn) {
        self.check_block(&f.block, f.sig.ident.span());
        syn::visit::visit_impl_item_fn(self, f);
    }
}

impl<'a> V<'a> {
    fn check_block(&mut self, block: &syn::Block, fn_span: proc_macro2::Span) {
        let mut saw_external = false;
        for stmt in &block.stmts {
            let t = stmt_text(stmt);
            if saw_external && is_state_write(&t) {
                let (line, column) = span_loc(fn_span);
                let snippet = snippet_for(self.path, line);
                self.out.push(Finding {
                    id: "SS004",
                    name: "state_write_after_external_call",
                    description:
                        "storage mutation observed after an external invoke_contract/Client call",
                    severity: Severity::Medium,
                    file: self.path.to_path_buf(),
                    line,
                    column,
                    snippet,
                    note: Some(
                        "Apply the Checks-Effects-Interactions pattern: finalize storage updates \
                         before handing off control to another contract. If the external call is \
                         trusted, document the assumption explicitly."
                            .into(),
                    ),
                });
                // Only one finding per function to avoid noise.
                return;
            }
            if is_external_call(&t) {
                saw_external = true;
            }
        }
    }
}
