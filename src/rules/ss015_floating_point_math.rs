//! SS015 — floating-point types in on-chain contract code.
//!
//! Soroban contracts execute across all validating nodes. Floating-point
//! arithmetic (`f32`, `f64`) is not bit-for-bit portable across compilers,
//! CPU feature sets (SSE vs. AVX rounding flags, FMA fusing), or Wasm
//! runtime vendors. A contract that uses `f32`/`f64` either (a) breaks
//! consensus when two validators disagree on a rounding step, or (b)
//! has been silently rewritten by the SDK author to avoid floats at the
//! boundary, in which case the internal use is dead weight that should
//! still be removed.
//!
//! The correct abstraction is fixed-point integers (`i128`, `u128`,
//! `soroban_sdk::U256`) with an explicit decimal scale.
//!
//! Detection, two axes:
//! 1. **Type path** — any `f32` / `f64` / `f128` as a type (parameter,
//!    return, local binding, generic arg, tuple element, inside
//!    `Vec<..>` / `Option<..>` / `Result<..>` etc.).
//! 2. **Cast expression** — `x as f32` / `x as f64` / `x as f128`.
//!
//! Both fire regardless of whether the function is inside an
//! `#[contractimpl]` block: in audit practice, float helpers live in
//! plain `mod` blocks and are called from `#[contractimpl]` methods, so
//! restricting the check to the impl block would miss them. The
//! negative fixture documents the intended escape hatch: helper modules
//! that are explicitly `#[cfg(test)]` are skipped by the scanner
//! upstream (see `ScanConfig::include_tests`), so `f64` inside a
//! `#[cfg(test)] mod tests` is already excluded.
//!
//! Severity: **High** — this is a hard consensus-safety issue, not a
//! stylistic one.

use super::Rule;
use crate::finding::{Finding, Severity};
use crate::walker::{snippet_for, span_loc};
use std::path::Path;
use syn::spanned::Spanned;
use syn::visit::Visit;
use syn::{ExprCast, File, TypePath};

pub struct Rule015;

impl Rule for Rule015 {
    fn id(&self) -> &'static str { "SS015" }
    fn name(&self) -> &'static str { "floating_point_math" }
    fn description(&self) -> &'static str {
        "floating-point type (f32/f64/f128) used in contract code — breaks consensus determinism"
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

fn is_float_ident(name: &str) -> bool {
    matches!(name, "f32" | "f64" | "f128")
}

impl<'a> V<'a> {
    fn push(&mut self, line: usize, column: usize, which: &'static str, context: &'static str) {
        let snippet = snippet_for(self.path, line);
        self.out.push(Finding {
            id: "SS015",
            name: "floating_point_math",
            description: "floating-point type used in contract code — breaks consensus determinism",
            severity: Severity::High,
            file: self.path.to_path_buf(),
            line,
            column,
            snippet,
            note: Some(
                format!(
                    "`{which}` appears in {context}. Floating-point arithmetic is not \
                     bit-for-bit portable across Wasm runtimes and CPU feature sets, so two \
                     validators can disagree on the result and halt consensus. Replace with \
                     a fixed-point integer representation (`i128`/`u128` with an explicit \
                     decimal scale, or `soroban_sdk::U256`)."
                ),
            ),
        });
    }
}

impl<'ast, 'a> Visit<'ast> for V<'a> {
    fn visit_type_path(&mut self, tp: &'ast TypePath) {
        // Leaf-level path like `f32`, `f64`, `f128`. We intentionally only
        // match when the final segment is a float ident; this catches
        // `f64`, `std::primitive::f64`, etc., but not an unrelated trait
        // named `SomeF64Thing`.
        if let Some(last) = tp.path.segments.last() {
            let n = last.ident.to_string();
            if is_float_ident(&n) {
                let (line, column) = span_loc(last.ident.span());
                let which: &'static str = match n.as_str() {
                    "f32" => "f32",
                    "f64" => "f64",
                    "f128" => "f128",
                    _ => "float",
                };
                self.push(line, column, which, "a type position");
            }
        }
        syn::visit::visit_type_path(self, tp);
    }

    fn visit_expr_cast(&mut self, c: &'ast ExprCast) {
        // `x as f64` — the `ty` is also visited by `visit_type_path`, so
        // the type-path arm would already fire. We explicitly report the
        // cast on its own line too, because the cast span tends to be
        // more informative than the bare type span when a user reads the
        // finding list.
        if let syn::Type::Path(tp) = &*c.ty {
            if let Some(last) = tp.path.segments.last() {
                let n = last.ident.to_string();
                if is_float_ident(&n) {
                    let (line, column) = span_loc(c.span());
                    let which: &'static str = match n.as_str() {
                        "f32" => "f32",
                        "f64" => "f64",
                        "f128" => "f128",
                        _ => "float",
                    };
                    self.push(line, column, which, "an `as`-cast expression");
                    // Skip walking the inner type-path so we don't
                    // double-report this exact site.
                    syn::visit::visit_expr(self, &c.expr);
                    return;
                }
            }
        }
        syn::visit::visit_expr_cast(self, c);
    }
}
