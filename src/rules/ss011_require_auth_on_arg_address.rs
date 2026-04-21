//! SS011 — complement to SS002: a contract function auths SOME but not all
//! of its `Address` parameters. This often happens when a refactor adds a
//! second counterparty (e.g. `on_behalf_of`, `recipient`, `delegate`) but
//! forgets to auth it.

use super::helpers::{address_param_names, for_each_contractimpl_fn};
use super::Rule;
use crate::finding::{Finding, Severity};
use crate::walker::{snippet_for, span_loc};
use std::path::Path;
use syn::visit::Visit;
use syn::File;

pub struct Rule011;

impl Rule for Rule011 {
    fn id(&self) -> &'static str { "SS011" }
    fn name(&self) -> &'static str { "partial_auth_coverage" }
    fn description(&self) -> &'static str {
        "function authenticates only a subset of its Address parameters"
    }

    fn run(&self, path: &Path, file: &File, _src: &str, out: &mut Vec<Finding>) {
        for_each_contractimpl_fn(file, |_imp, fun| {
            let params = address_param_names(&fun.sig);
            if params.len() < 2 {
                return;
            }
            let mut finder = Finder::default();
            finder.visit_block(&fun.block);
            let mut authed = finder.authed;
            authed.sort();
            authed.dedup();

            // If nothing is authed, SS002 already reported. We only care about
            // partial coverage.
            let covered: Vec<_> = params.iter().filter(|p| authed.contains(p)).cloned().collect();
            if covered.is_empty() || covered.len() == params.len() {
                return;
            }
            let missing: Vec<_> = params.iter().filter(|p| !authed.contains(p)).cloned().collect();

            let (line, column) = span_loc(fun.sig.ident.span());
            let snippet = snippet_for(path, line);
            out.push(Finding {
                id: "SS011",
                name: "partial_auth_coverage",
                description: "partial Address auth coverage in contract function",
                severity: Severity::Medium,
                file: path.to_path_buf(),
                line,
                column,
                snippet,
                note: Some(format!(
                    "Authenticated: {covered:?}. NOT authenticated: {missing:?}. Either auth \
                     all parties (safer) or add a comment justifying why the un-auth'd party \
                     is trusted."
                )),
            });
        });
    }
}

#[derive(Default)]
struct Finder {
    authed: Vec<String>,
}

impl<'ast> Visit<'ast> for Finder {
    fn visit_expr_method_call(&mut self, mc: &'ast syn::ExprMethodCall) {
        if mc.method == "require_auth" || mc.method == "require_auth_for_args" {
            let receiver_text = quote::quote!(#(&mc.receiver)).to_string();
            // Extract the last identifier in the receiver text.
            if let Some(ident) = receiver_text
                .split(|c: char| !c.is_ascii_alphanumeric() && c != '_')
                .rfind(|s| !s.is_empty())
            {
                self.authed.push(ident.to_string());
            }
        }
        syn::visit::visit_expr_method_call(self, mc);
    }
}
