//! SS009 — `(a / b) * c` pattern (division before multiplication) in
//! financial math, causing precision loss that compounds with every
//! operation.

use super::Rule;
use crate::finding::{Finding, Severity};
use crate::walker::{snippet_for, span_loc};
use std::path::Path;
use syn::spanned::Spanned;
use syn::visit::Visit;
use syn::{BinOp, Expr, File};

pub struct Rule009;

impl Rule for Rule009 {
    fn id(&self) -> &'static str { "SS009" }
    fn name(&self) -> &'static str { "division_before_multiplication" }
    fn description(&self) -> &'static str {
        "`(a / b) * c` loses precision; rearrange as `(a * c) / b` unless overflow is a concern"
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

fn is_div(e: &Expr) -> bool {
    if let Expr::Binary(b) = e {
        matches!(b.op, BinOp::Div(_))
    } else if let Expr::Paren(p) = e {
        is_div(&p.expr)
    } else {
        false
    }
}

impl<'ast, 'a> Visit<'ast> for V<'a> {
    fn visit_expr(&mut self, e: &'ast Expr) {
        if let Expr::Binary(bin) = e {
            if matches!(bin.op, BinOp::Mul(_)) && (is_div(&bin.left) || is_div(&bin.right)) {
                let (line, column) = span_loc(bin.op.span());
                let snippet = snippet_for(self.path, line);
                self.out.push(Finding {
                    id: "SS009",
                    name: "division_before_multiplication",
                    description: "multiplication follows a division — rearrange to preserve precision",
                    severity: Severity::Low,
                    file: self.path.to_path_buf(),
                    line,
                    column,
                    snippet,
                    note: Some(
                        "Integer math truncates on each `/`. For price/interest calculations \
                         prefer `(a * c) / b` or switch to U256 intermediates."
                            .into(),
                    ),
                });
            }
        }
        syn::visit::visit_expr(self, e);
    }
}
