//! SS006 — writes to `env.storage().persistent()` that never pair with a
//! matching `extend_ttl(...)` in the same function. Persistent entries have
//! a bounded TTL; if writers don't bump it, entries silently archive and the
//! protocol starts returning defaults from reads (see SS001).
//!
//! Heuristic: per function, count `persistent().set(...)` and
//! `persistent().extend_ttl(...)` (or `.bump(...)`) calls. If writes happen
//! but no TTL maintenance, emit a Low/Medium finding.

use super::Rule;
use crate::finding::{Finding, Severity};
use crate::walker::{snippet_for, span_loc};
use std::path::Path;
use syn::visit::Visit;
use syn::File;

pub struct Rule006;

impl Rule for Rule006 {
    fn id(&self) -> &'static str { "SS006" }
    fn name(&self) -> &'static str { "persistent_write_without_extend_ttl" }
    fn description(&self) -> &'static str {
        "function writes to persistent storage but never extends the entry's TTL; entries may archive and be read back as defaults"
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

impl<'a> V<'a> {
    fn check(&mut self, block: &syn::Block, sig_span: proc_macro2::Span) {
        let text = quote::quote!(#block).to_string();
        let has_persistent_set = text.contains("persistent") && text.contains(". set (");
        let has_extend = text.contains("extend_ttl") || text.contains(".bump(");
        if has_persistent_set && !has_extend {
            let (line, column) = span_loc(sig_span);
            let snippet = snippet_for(self.path, line);
            self.out.push(Finding {
                id: "SS006",
                name: "persistent_write_without_extend_ttl",
                description: "persistent().set(...) without a matching extend_ttl/bump",
                severity: Severity::Low,
                file: self.path.to_path_buf(),
                line,
                column,
                snippet,
                note: Some(
                    "Call `env.storage().persistent().extend_ttl(&key, MIN, EXTEND)` after \
                     each set, or centralize TTL maintenance. Skipping it means the entry \
                     will eventually archive and downstream reads may collapse to defaults \
                     (see SS001)."
                        .into(),
                ),
            });
        }
    }
}

impl<'ast, 'a> Visit<'ast> for V<'a> {
    fn visit_item_fn(&mut self, f: &'ast syn::ItemFn) {
        self.check(&f.block, f.sig.ident.span());
    }
    fn visit_impl_item_fn(&mut self, f: &'ast syn::ImplItemFn) {
        self.check(&f.block, f.sig.ident.span());
    }
}
