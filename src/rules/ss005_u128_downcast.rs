//! SS005 — downcast from a wider integer (U256, i256, u128) to a narrower
//! type without a bounds check. Soroban lending protocols perform mixed-
//! decimal math (balance × price × oracle_to_wad / decimals) using U256
//! intermediates; a silent truncation silently mints / burns value.

use super::Rule;
use crate::finding::{Finding, Severity};
use crate::walker::{snippet_for, span_loc};
use std::path::Path;
use syn::visit::Visit;
use syn::{Expr, File};

pub struct Rule005;

impl Rule for Rule005 {
    fn id(&self) -> &'static str { "SS005" }
    fn name(&self) -> &'static str { "wide_to_narrow_downcast" }
    fn description(&self) -> &'static str {
        "`.to_u128()` / `.to_u64()` / `as u128` on a wider integer truncates silently if the value overflows"
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
    fn visit_expr(&mut self, e: &'ast Expr) {
        match e {
            Expr::MethodCall(mc) => {
                let narrow = matches!(
                    mc.method.to_string().as_str(),
                    "to_u128" | "to_u64" | "to_u32" | "to_i128" | "to_i64"
                );
                if narrow {
                    let (line, column) = span_loc(mc.method.span());
                    let snippet = snippet_for(self.path, line);
                    self.out.push(Finding {
                        id: "SS005",
                        name: "wide_to_narrow_downcast",
                        description: "explicit downcast method without overflow check",
                        severity: Severity::Medium,
                        file: self.path.to_path_buf(),
                        line,
                        column,
                        snippet,
                        note: Some(
                            "Prefer `.try_into()?` or check bounds explicitly before converting. \
                             Silent truncation on value × price calculations can allow minting \
                             arbitrary balances."
                                .into(),
                        ),
                    });
                }
            }
            Expr::Cast(c) => {
                // `x as u128`, `x as u64`, etc.
                if let syn::Type::Path(tp) = &*c.ty {
                    if let Some(seg) = tp.path.segments.last() {
                        let n = seg.ident.to_string();
                        if matches!(n.as_str(), "u128" | "u64" | "u32" | "i128" | "i64" | "u16" | "u8") {
                            let (line, column) = span_loc(c.as_token.span);
                            let snippet = snippet_for(self.path, line);
                            self.out.push(Finding {
                                id: "SS005",
                                name: "wide_to_narrow_downcast",
                                description: "`as`-cast to narrower integer type may truncate",
                                severity: Severity::Low,
                                file: self.path.to_path_buf(),
                                line,
                                column,
                                snippet,
                                note: Some(
                                    "`as` casts truncate silently. For financial math, use \
                                     `TryFrom`/`try_into` and handle the error."
                                        .into(),
                                ),
                            });
                        }
                    }
                }
            }
            _ => {}
        }
        syn::visit::visit_expr(self, e);
    }
}
