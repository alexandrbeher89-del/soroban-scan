//! Helpers shared by multiple rules.
//!
//! These are deliberately best-effort heuristics: soroban-scan is an assist,
//! not a theorem prover, and callers are expected to triage reported findings.
#![allow(dead_code)]

use syn::{Expr, ExprMethodCall, ImplItem, Item, Pat, Path, ReturnType, Type};

/// Return true if the path ends with `name` as its last segment.
pub fn path_ends_with(path: &Path, name: &str) -> bool {
    path.segments.last().map(|s| s.ident == name).unwrap_or(false)
}

/// Whether `m` is `<recv>.<name>(...)`.
pub fn method_is(m: &ExprMethodCall, name: &str) -> bool {
    m.method == name
}

/// Walks the chain `a.b().c().d()` and returns the top-most receiver expression
/// plus the ordered list of method names seen (in call-order: `[b, c, d]`).
pub fn method_chain(expr: &Expr) -> (Option<&Expr>, Vec<String>) {
    let mut names = Vec::new();
    let mut cur = expr;
    while let Expr::MethodCall(mc) = cur {
        names.push(mc.method.to_string());
        cur = &mc.receiver;
    }
    names.reverse();
    (Some(cur), names)
}

/// True if the chain contains a method named `name` anywhere.
pub fn chain_contains(expr: &Expr, name: &str) -> bool {
    let (_, names) = method_chain(expr);
    names.iter().any(|n| n == name)
}

/// Extract the identifier part of the last segment of a type path, if any.
pub fn type_last_ident(ty: &Type) -> Option<String> {
    if let Type::Path(tp) = ty {
        tp.path.segments.last().map(|s| s.ident.to_string())
    } else {
        None
    }
}

/// True if `ty` is the Soroban `Address` type (by unqualified match — we can't
/// always resolve the full path, but this is accurate enough in practice
/// because Soroban contracts conventionally `use soroban_sdk::Address`).
pub fn is_address_type(ty: &Type) -> bool {
    matches!(type_last_ident(ty).as_deref(), Some("Address"))
}

/// Iterate over all `impl` blocks marked `#[contractimpl]` in a file and yield
/// their `ImplItem`s paired with the self-type name.
pub fn for_each_contractimpl_fn<'a, F>(file: &'a syn::File, mut f: F)
where
    F: FnMut(&'a syn::ItemImpl, &'a syn::ImplItemFn),
{
    for item in &file.items {
        let Item::Impl(imp) = item else { continue };
        let has_attr = imp
            .attrs
            .iter()
            .any(|a| path_ends_with(a.path(), "contractimpl"));
        if !has_attr {
            continue;
        }
        for ii in &imp.items {
            if let ImplItem::Fn(fun) = ii {
                f(imp, fun);
            }
        }
    }
}

/// Return `true` if the signature has an `Address` parameter at any position
/// (other than `&self` / `&mut self`).
pub fn has_address_param(sig: &syn::Signature) -> bool {
    sig.inputs.iter().any(|inp| {
        if let syn::FnArg::Typed(pat) = inp {
            is_address_type(&pat.ty)
        } else {
            false
        }
    })
}

/// Collect the names of `Address`-typed parameters of a function (other than receiver).
pub fn address_param_names(sig: &syn::Signature) -> Vec<String> {
    let mut names = Vec::new();
    for inp in sig.inputs.iter() {
        if let syn::FnArg::Typed(pat) = inp {
            if is_address_type(&pat.ty) {
                if let Pat::Ident(pi) = &*pat.pat {
                    names.push(pi.ident.to_string());
                }
            }
        }
    }
    names
}

/// Whether a function returns a `Result<_, _>` (which is Soroban-idiomatic).
pub fn returns_result(sig: &syn::Signature) -> bool {
    if let ReturnType::Type(_, ty) = &sig.output {
        if let Type::Path(tp) = &**ty {
            return tp
                .path
                .segments
                .last()
                .map(|s| s.ident == "Result")
                .unwrap_or(false);
        }
    }
    false
}
