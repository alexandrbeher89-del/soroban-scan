//! SS006 — writes to `env.storage().persistent()` that never pair with a
//! matching `extend_ttl(...)` in the same function.
//!
//! ### Post CAP-0066 scope (Protocol 23)
//!
//! This is **not** a state-integrity issue. CAP-0066 auto-restores archived
//! persistent entries on `InvokeHostFunctionOp`, so contract code will never
//! read an archived entry as a default value. Missing `extend_ttl` only
//! translates into:
//!
//! 1. The **user** paying a restoration fee on the next interaction that
//!    touches the entry (can also cause tx fee-budget failures).
//! 2. Worst case: the entry is evicted and a later user has to pay a larger
//!    restoration, but the written value is preserved.
//!
//! Keep this rule as an **Info**-level UX/cost hint, not a security finding.
//!
//! Heuristic: per function, look for `persistent().set(...)` with no
//! `extend_ttl(...)`/`.bump(...)` in the same function body.

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
        "persistent().set(...) without a matching extend_ttl(...); eviction is harmless under CAP-0066 but imposes a restoration fee on the next caller"
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
                description: "persistent().set(...) without a matching extend_ttl/bump (restoration-fee hint)",
                severity: Severity::Info,
                file: self.path.to_path_buf(),
                line,
                column,
                snippet,
                note: Some(
                    "Under CAP-0066 (Protocol 23), evicted persistent entries are automatically \
                     restored when next touched, so the stored value is preserved. The only \
                     downside of a missing extend_ttl is that the next caller pays a restoration \
                     fee (and may hit the tx fee budget). If this is a hot-path entry, add \
                     `env.storage().persistent().extend_ttl(&key, MIN, EXTEND)` to amortize the \
                     rent cost onto the writer."
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
