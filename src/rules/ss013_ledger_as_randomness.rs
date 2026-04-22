//! SS013 — `env.ledger().timestamp()` or `env.ledger().sequence()` used as a
//! source of "randomness" (typically via `% N` or as a seed). Ledger state is
//! fully deterministic and public before a contract runs; using it as entropy
//! is a classic Soroban anti-pattern that lets callers / sequencers trivially
//! grind the result.
//!
//! Detection: binary expression with `%` (or `/`) operator whose **left-hand
//! side contains a call to `timestamp()` or `sequence()` reached through a
//! `ledger()` call**. Also flags direct use of those values as an argument to
//! any function whose name contains `rand`, `seed`, `shuffle`, `draw`,
//! `entropy`.
//!
//! Examples flagged:
//! ```text
//!     let r = env.ledger().timestamp() % 100;
//!     let pick = (e.ledger().sequence() as u128) % self.winners.len();
//!     let seed = seed_from(env.ledger().timestamp());
//! ```

use super::Rule;
use crate::finding::{Finding, Severity};
use crate::walker::{snippet_for, span_loc};
use std::path::Path;
use syn::spanned::Spanned;
use syn::visit::Visit;
use syn::{BinOp, Expr, ExprMethodCall, File};

pub struct Rule013;

impl Rule for Rule013 {
    fn id(&self) -> &'static str { "SS013" }
    fn name(&self) -> &'static str { "ledger_as_randomness" }
    fn description(&self) -> &'static str {
        "env.ledger().timestamp() / .sequence() used as entropy (modulo, seed, shuffle source)"
    }

    fn run(&self, path: &Path, file: &File, _src: &str, out: &mut Vec<Finding>) {
        let mut v = V { path, out, seen: std::collections::HashSet::new() };
        v.visit_file(file);
    }
}

struct V<'a> {
    path: &'a Path,
    out: &'a mut Vec<Finding>,
    // Dedup by (line, column) so a `%` + surrounding call both firing on the
    // same token don't produce two findings.
    seen: std::collections::HashSet<(usize, usize)>,
}

/// Return `Some(kind)` if `expr` (recursively) contains a method call chain
/// ending in `timestamp()` or `sequence()` whose receiver transitively calls
/// `ledger()`. `kind` is `"timestamp"` or `"sequence"`.
fn ledger_source(expr: &Expr) -> Option<&'static str> {
    // Look for .timestamp() / .sequence()
    if let Expr::MethodCall(mc) = expr {
        let m = mc.method.to_string();
        if (m == "timestamp" || m == "sequence") && receiver_calls_ledger(&mc.receiver) {
            return Some(if m == "timestamp" { "timestamp" } else { "sequence" });
        }
    }
    // Walk through common wrapping: casts, parentheses, unary, binary lhs.
    match expr {
        Expr::Paren(p) => ledger_source(&p.expr),
        Expr::Cast(c) => ledger_source(&c.expr),
        Expr::Unary(u) => ledger_source(&u.expr),
        Expr::Binary(b) => ledger_source(&b.left).or_else(|| ledger_source(&b.right)),
        Expr::MethodCall(mc) => ledger_source(&mc.receiver),
        Expr::Reference(r) => ledger_source(&r.expr),
        _ => None,
    }
}

/// Check whether this expression is (or contains) a call `.ledger()`.
fn receiver_calls_ledger(expr: &Expr) -> bool {
    match expr {
        Expr::MethodCall(mc) => {
            if mc.method == "ledger" {
                return true;
            }
            receiver_calls_ledger(&mc.receiver)
        }
        Expr::Paren(p) => receiver_calls_ledger(&p.expr),
        Expr::Reference(r) => receiver_calls_ledger(&r.expr),
        _ => false,
    }
}

/// Treat the identifier as entropy-flavored only when one of the keywords
/// appears as a **whole word-segment**, not a substring. This avoids flagging
/// DeFi names like `withdraw` (contains "draw"), `brand` (contains "rand"),
/// `operand`, `strand`, etc.
///
/// Segments are split on `_`; a keyword matches if it equals a segment
/// exactly, OR (for the multi-letter keywords) is a prefix/suffix of the
/// segment joined by `_` boundaries (so `rng_seed`, `seed_from`, `randomize`
/// still match). `randomize` is handled by allowing prefix-match on
/// `random`/`rand`.
fn ident_looks_random(name: &str) -> bool {
    let n = name.to_lowercase();
    // Single-word cheap check first.
    const EXACT: &[&str] = &[
        "rand", "random", "randomize",
        "seed",
        "shuffle",
        "draw",
        "entropy",
        "rng", "prng",
    ];
    for seg in n.split('_') {
        if EXACT.contains(&seg) {
            return true;
        }
        // Prefix forms like `random_*`, `seed_*`, `shuffle_*`.
        if seg.starts_with("random") || seg.starts_with("shuffle") || seg.starts_with("entropy") {
            return true;
        }
    }
    // Whole-name prefixes not caught by segmenting: `seed_from`, `rand_from`
    // are already caught above; `randomize` is caught by `starts_with`.
    false
}

fn call_name_looks_random(mc: &ExprMethodCall) -> bool {
    ident_looks_random(&mc.method.to_string())
}

fn path_last_looks_random(p: &syn::Path) -> bool {
    p.segments
        .last()
        .map(|s| ident_looks_random(&s.ident.to_string()))
        .unwrap_or(false)
}

impl<'ast, 'a> Visit<'ast> for V<'a> {
    fn visit_expr(&mut self, e: &'ast Expr) {
        // Case 1: modulo / division with ledger source on either side.
        if let Expr::Binary(bin) = e {
            let is_div_or_mod = matches!(
                bin.op,
                BinOp::Rem(_) | BinOp::RemAssign(_) | BinOp::Div(_) | BinOp::DivAssign(_)
            );
            if is_div_or_mod {
                if let Some(kind) = ledger_source(&bin.left).or_else(|| ledger_source(&bin.right)) {
                    let (line, column) = span_loc(bin.op.span());
                    if self.seen.insert((line, column)) {
                        let op = if matches!(bin.op, BinOp::Rem(_) | BinOp::RemAssign(_)) { "%" } else { "/" };
                        self.out.push(Finding {
                            id: "SS013",
                            name: "ledger_as_randomness",
                            description:
                                "ledger timestamp/sequence used as entropy",
                            severity: Severity::Medium,
                            file: self.path.to_path_buf(),
                            line,
                            column,
                            snippet: snippet_for(self.path, line),
                            note: Some(format!(
                                "`env.ledger().{kind}() {op} N` — ledger state is deterministic and \
                                 publicly observable before your contract runs, so callers or \
                                 sequencers can grind this value. If you need randomness, commit \
                                 to a future-blockhash / VRF scheme instead."
                            )),
                        });
                    }
                }
            }
        }

        // Case 2: ledger source passed into a function / method whose name
        // suggests randomness (`rand_from`, `seed_from`, `shuffle_with`, …).
        if let Expr::Call(call) = e {
            if let Expr::Path(p) = &*call.func {
                if path_last_looks_random(&p.path) {
                    for arg in &call.args {
                        if let Some(kind) = ledger_source(arg) {
                            let (line, column) = span_loc(call.span());
                            if self.seen.insert((line, column)) {
                                self.out.push(Finding {
                                    id: "SS013",
                                    name: "ledger_as_randomness",
                                    description: "ledger value fed into a random/seed/shuffle function",
                                    severity: Severity::Medium,
                                    file: self.path.to_path_buf(),
                                    line,
                                    column,
                                    snippet: snippet_for(self.path, line),
                                    note: Some(format!(
                                        "`env.ledger().{kind}()` passed to a function named like a \
                                         random/seed/shuffle primitive. Ledger values are \
                                         deterministic; use commit-reveal or VRF instead."
                                    )),
                                });
                                break;
                            }
                        }
                    }
                }
            }
        }
        if let Expr::MethodCall(mc) = e {
            if call_name_looks_random(mc) {
                for arg in &mc.args {
                    if let Some(kind) = ledger_source(arg) {
                        let (line, column) = span_loc(mc.span());
                        if self.seen.insert((line, column)) {
                            self.out.push(Finding {
                                id: "SS013",
                                name: "ledger_as_randomness",
                                description: "ledger value fed into a random/seed/shuffle method",
                                severity: Severity::Medium,
                                file: self.path.to_path_buf(),
                                line,
                                column,
                                snippet: snippet_for(self.path, line),
                                note: Some(format!(
                                    "`env.ledger().{kind}()` passed to `.{}(…)`. Ledger values \
                                     are deterministic; use commit-reveal or VRF instead.",
                                    mc.method
                                )),
                            });
                            break;
                        }
                    }
                }
            }
        }

        syn::visit::visit_expr(self, e);
    }
}
