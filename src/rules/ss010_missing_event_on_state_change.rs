//! SS010 — a `#[contractimpl]` function writes to storage but emits no
//! event. This hurts indexers / off-chain consumers and is often a signal
//! that the function's state change path is incomplete.

use super::helpers::for_each_contractimpl_fn;
use super::Rule;
use crate::finding::{Finding, Severity};
use crate::walker::{snippet_for, span_loc};
use std::path::Path;
use syn::File;

pub struct Rule010;

impl Rule for Rule010 {
    fn id(&self) -> &'static str { "SS010" }
    fn name(&self) -> &'static str { "missing_event_on_state_change" }
    fn description(&self) -> &'static str {
        "contract method mutates storage but emits no event — observers can miss the state change"
    }

    fn run(&self, path: &Path, file: &File, _src: &str, out: &mut Vec<Finding>) {
        for_each_contractimpl_fn(file, |_imp, fun| {
            let name = fun.sig.ident.to_string();
            if name.starts_with("get_")
                || name.starts_with("query_")
                || name.starts_with("view_")
                || name.starts_with("has_")
                || name.starts_with("is_")
            {
                return;
            }
            let body = quote::quote!(#(&fun.block)).to_string();
            let writes = body.contains(". set (") || body.contains(". update (") || body.contains(". remove (");
            let emits = body.contains("events().publish") || body.contains(".publish(");
            if writes && !emits {
                let (line, column) = span_loc(fun.sig.ident.span());
                let snippet = snippet_for(path, line);
                out.push(Finding {
                    id: "SS010",
                    name: "missing_event_on_state_change",
                    description:
                        "public contract function writes to storage but does not publish an event",
                    severity: Severity::Info,
                    file: path.to_path_buf(),
                    line,
                    column,
                    snippet,
                    note: Some(
                        "Emit an event with `env.events().publish((topic,), data)` after each \
                         state change so indexers and UIs can observe it."
                            .into(),
                    ),
                });
            }
        });
    }
}
