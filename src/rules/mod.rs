//! Rule registry.
//!
//! Each rule implements [`Rule`] and is listed in [`all_rules`]. Rules are
//! intentionally independent and side-effect free so they can be run in any
//! order and, eventually, in parallel.

use crate::finding::Finding;
use std::path::Path;
use syn::File;

pub trait Rule: Send + Sync {
    fn id(&self) -> &'static str;
    fn name(&self) -> &'static str;
    fn description(&self) -> &'static str;
    /// Visit a parsed file and push any findings onto `out`.
    fn run(&self, path: &Path, file: &File, src: &str, out: &mut Vec<Finding>);
}

mod helpers;

mod ss001_unchecked_storage_get;
mod ss002_missing_require_auth;
mod ss003_unbounded_vec_iter;
mod ss004_invoke_contract_reentrancy;
mod ss005_u128_downcast;
mod ss006_persistent_without_extend_ttl;
mod ss007_storage_enum_collision;
mod ss008_bitmap_or_without_mask;
mod ss009_division_before_multiplication;
mod ss010_missing_event_on_state_change;
mod ss011_require_auth_on_arg_address;
mod ss012_panic_unwrap_in_contract;
mod ss013_ledger_as_randomness;
mod ss014_deprecated_bump_api;

pub fn all_rules() -> Vec<Box<dyn Rule>> {
    vec![
        Box::new(ss001_unchecked_storage_get::Rule001),
        Box::new(ss002_missing_require_auth::Rule002),
        Box::new(ss003_unbounded_vec_iter::Rule003),
        Box::new(ss004_invoke_contract_reentrancy::Rule004),
        Box::new(ss005_u128_downcast::Rule005),
        Box::new(ss006_persistent_without_extend_ttl::Rule006),
        Box::new(ss007_storage_enum_collision::Rule007),
        Box::new(ss008_bitmap_or_without_mask::Rule008),
        Box::new(ss009_division_before_multiplication::Rule009),
        Box::new(ss010_missing_event_on_state_change::Rule010),
        Box::new(ss011_require_auth_on_arg_address::Rule011),
        Box::new(ss012_panic_unwrap_in_contract::Rule012),
        Box::new(ss013_ledger_as_randomness::Rule013),
        Box::new(ss014_deprecated_bump_api::Rule014),
    ]
}
