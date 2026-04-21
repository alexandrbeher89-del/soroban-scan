//! SS007 — two variants of a `#[contracttype]` enum that produce the same
//! XDR-encoded storage key because they differ only by an explicit
//! discriminant collision OR because both are name-equal modulo case.
//!
//! This is a conservative lint: we flag duplicated *names* (case-insensitive)
//! and #[derive(...)]-generated enums that use manual discriminants where
//! two variants carry the same discriminant. In practice, the Soroban SDK
//! encodes variant name + fields; if a refactor renames one variant but the
//! on-chain data still has the old name, storage collides — so duplicate
//! names across a file are worth flagging.

use super::Rule;
use crate::finding::{Finding, Severity};
use crate::walker::{snippet_for, span_loc};
use std::collections::HashMap;
use std::path::Path;
use syn::{File, Item};

pub struct Rule007;

impl Rule for Rule007 {
    fn id(&self) -> &'static str { "SS007" }
    fn name(&self) -> &'static str { "storage_enum_key_collision" }
    fn description(&self) -> &'static str {
        "#[contracttype] enum variants with colliding discriminants / names — risk of storage key collision"
    }

    fn run(&self, path: &Path, file: &File, _src: &str, out: &mut Vec<Finding>) {
        for item in &file.items {
            let Item::Enum(en) = item else { continue };
            let is_contracttype = en.attrs.iter().any(|a| {
                a.path().segments.last().map(|s| s.ident == "contracttype").unwrap_or(false)
            });
            if !is_contracttype {
                continue;
            }

            // Look for duplicate discriminants.
            let mut discs: HashMap<String, Vec<String>> = HashMap::new();
            for v in &en.variants {
                if let Some((_, expr)) = &v.discriminant {
                    let t = quote::quote!(#expr).to_string();
                    discs.entry(t).or_default().push(v.ident.to_string());
                }
            }
            for (disc, variants) in &discs {
                if variants.len() > 1 {
                    let (line, column) = span_loc(en.ident.span());
                    let snippet = snippet_for(path, line);
                    out.push(Finding {
                        id: "SS007",
                        name: "storage_enum_key_collision",
                        description:
                            "multiple #[contracttype] enum variants share the same discriminant",
                        severity: Severity::High,
                        file: path.to_path_buf(),
                        line,
                        column,
                        snippet,
                        note: Some(format!(
                            "Enum `{}` variants {:?} all use discriminant `{}`. XDR-encoded \
                             storage keys will alias and reads/writes will corrupt each other.",
                            en.ident, variants, disc,
                        )),
                    });
                }
            }

            // Case-insensitive duplicate names (unlikely but cheap).
            let mut lower = HashMap::new();
            for v in &en.variants {
                let k = v.ident.to_string().to_lowercase();
                lower.entry(k).or_insert_with(Vec::new).push(v.ident.to_string());
            }
            for (_, names) in lower {
                if names.len() > 1 {
                    let (line, column) = span_loc(en.ident.span());
                    let snippet = snippet_for(path, line);
                    out.push(Finding {
                        id: "SS007",
                        name: "storage_enum_key_collision",
                        description:
                            "#[contracttype] enum has variants whose names differ only by case",
                        severity: Severity::Medium,
                        file: path.to_path_buf(),
                        line,
                        column,
                        snippet,
                        note: Some(format!("Variants: {names:?}. Review XDR encoding.")),
                    });
                }
            }
        }
    }
}
