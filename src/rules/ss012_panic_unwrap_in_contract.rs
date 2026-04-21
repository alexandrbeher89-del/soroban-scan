//! SS012 — `.unwrap()` / `.expect(...)` / `panic!()` inside a `#[contractimpl]`
//! method. Panics in Soroban convert to opaque "Error(Contract, #N)" frame
//! with no stable code; auditors and users get worse UX and judges often
//! reward this as Low/QA. Prefer `Result<_, Error>` with explicit error codes.

use super::helpers::for_each_contractimpl_fn;
use super::Rule;
use crate::finding::{Finding, Severity};
use crate::walker::snippet_for;
use std::path::Path;
use syn::visit::Visit;
use syn::{Expr, File};

pub struct Rule012;

impl Rule for Rule012 {
    fn id(&self) -> &'static str { "SS012" }
    fn name(&self) -> &'static str { "panic_in_contract_function" }
    fn description(&self) -> &'static str {
        "contract function uses .unwrap() / .expect() / panic! — prefer typed contract Error"
    }

    fn run(&self, path: &Path, file: &File, _src: &str, out: &mut Vec<Finding>) {
        for_each_contractimpl_fn(file, |_imp, fun| {
            let mut finder = PanicFinder { hits: Vec::new() };
            finder.visit_block(&fun.block);
            // Only one finding per function, the first hit, to avoid noise.
            if let Some((line, column, kind)) = finder.hits.into_iter().next() {
                let snippet = snippet_for(path, line);
                out.push(Finding {
                    id: "SS012",
                    name: "panic_in_contract_function",
                    description: "panic / unwrap / expect inside a #[contractimpl] method",
                    severity: Severity::Low,
                    file: path.to_path_buf(),
                    line,
                    column,
                    snippet,
                    note: Some(format!(
                        "Found `{kind}`. Return `Result<_, YourError>` and `.map_err(...)` / \
                         `.ok_or(...)` instead; callers and auditors get a stable error code."
                    )),
                });
            }
        });
    }
}

struct PanicFinder {
    hits: Vec<(usize, usize, &'static str)>,
}

impl<'ast> Visit<'ast> for PanicFinder {
    fn visit_expr(&mut self, e: &'ast Expr) {
        if let Expr::MethodCall(mc) = e {
            let name = mc.method.to_string();
            if name == "unwrap" || name == "expect" {
                let (line, col) = crate::walker::span_loc(mc.method.span());
                let kind = if name == "unwrap" { "unwrap" } else { "expect" };
                self.hits.push((line, col, kind));
            }
        }
        if let Expr::Macro(mac) = e {
            let path = &mac.mac.path;
            if let Some(seg) = path.segments.last() {
                let n = seg.ident.to_string();
                if n == "panic" || n == "unreachable" || n == "todo" {
                    let (line, col) = crate::walker::span_loc(seg.ident.span());
                    let kind = match n.as_str() {
                        "panic" => "panic!",
                        "unreachable" => "unreachable!",
                        _ => "todo!",
                    };
                    self.hits.push((line, col, kind));
                }
            }
        }
        syn::visit::visit_expr(self, e);
    }
}
