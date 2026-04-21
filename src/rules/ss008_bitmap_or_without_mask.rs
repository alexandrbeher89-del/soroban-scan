//! SS008 — bitmap mutation `x |= mask` or `x = x | mask` on a user-config-
//! looking variable without first clearing the bit(s) being written. K2's
//! user-config bitmap uses two bits per reserve (collateral flag + borrow
//! flag); setting without clearing leaves stale bits.

use super::Rule;
use crate::finding::{Finding, Severity};
use crate::walker::{snippet_for, span_loc};
use std::path::Path;
use syn::spanned::Spanned;
use syn::visit::Visit;
use syn::{BinOp, Expr, File};

pub struct Rule008;

impl Rule for Rule008 {
    fn id(&self) -> &'static str { "SS008" }
    fn name(&self) -> &'static str { "bitmap_or_without_clear_mask" }
    fn description(&self) -> &'static str {
        "`|=` applied to a bitmap field without a matching `& !mask` earlier — may leave stale bits"
    }

    fn run(&self, path: &Path, file: &File, src: &str, out: &mut Vec<Finding>) {
        let mut v = V { path, src, out };
        v.visit_file(file);
    }
}

struct V<'a> {
    path: &'a Path,
    src: &'a str,
    out: &'a mut Vec<Finding>,
}

fn looks_like_bitmap(text: &str) -> bool {
    let t = text.to_lowercase();
    t.contains("bitmap")
        || t.contains("config")
        || t.contains("flags")
        || t.contains("user_configuration")
        || t.contains("data")
}

impl<'ast, 'a> Visit<'ast> for V<'a> {
    fn visit_expr(&mut self, e: &'ast Expr) {
        if let Expr::Binary(bin) = e {
            if matches!(bin.op, BinOp::BitOrAssign(_)) {
                let lhs_text = quote::quote!(#(&bin.left)).to_string();
                let rhs_text = quote::quote!(#(&bin.right)).to_string();
                if looks_like_bitmap(&lhs_text) {
                    // Cheap check: before this op in the same file's source,
                    // does any line contain "& !" or ".clear_bit" near the
                    // lhs name?
                    let lhs_ident = lhs_text.trim();
                    let short = lhs_ident.split_whitespace().last().unwrap_or(lhs_ident);
                    let clear_nearby =
                        self.src.contains(&format!("{short} & !"))
                            || self.src.contains(&format!("{short} &= !"))
                            || self.src.contains("clear_bit")
                            || rhs_text.contains("| ");
                    if !clear_nearby {
                        let (line, column) = span_loc(bin.op.span());
                        let snippet = snippet_for(self.path, line);
                        self.out.push(Finding {
                            id: "SS008",
                            name: "bitmap_or_without_clear_mask",
                            description:
                                "bitwise OR-assignment on a bitmap without first clearing the relevant bits",
                            severity: Severity::Low,
                            file: self.path.to_path_buf(),
                            line,
                            column,
                            snippet,
                            note: Some(
                                "When updating multi-bit flags, ALWAYS `x &= !mask` before `x |= new_value`."
                                    .into(),
                            ),
                        });
                    }
                }
            }
        }
        syn::visit::visit_expr(self, e);
    }
}
