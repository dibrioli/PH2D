//! The engine hooks (`init`, `update`, …): luau-lsp does not know the engine calls them.

use super::{Diagnostic, LINT_FUNCTION_UNUSED};
use crate::position::position_to_byte;
use std::collections::BTreeSet;

/// The names of the `-- @hook <name>: <type>` lines of a definitions file — the global functions
/// the engine calls (generated from `ph2d_script::api_table::HOOKS`).
pub fn hooks_in_definitions(text: &str) -> Vec<String> {
    text.lines()
        .filter_map(|l| l.strip_prefix("-- @hook "))
        .filter_map(|rest| rest.split_once(':'))
        .map(|(name, _)| name.trim().to_owned())
        .filter(|n| !n.is_empty())
        .collect()
}

/// `FunctionUnused` on a GLOBAL `function <hook>` — the engine is its caller, invisible to luau-lsp.
/// By exact name and only for globals: a misspelt hook or a `local function update` still warns.
pub fn is_unused_engine_hook(d: &Diagnostic, text: &str, hooks: &BTreeSet<String>) -> bool {
    if d.code.as_deref() != Some(LINT_FUNCTION_UNUSED) || !d.message.starts_with("FunctionUnused:")
    {
        return false;
    }
    let (a, b) = (
        position_to_byte(text, d.range.start),
        position_to_byte(text, d.range.end),
    );
    let Some(name) = text.get(a..b) else {
        return false;
    };
    let line_start = text[..a].rfind(['\n', '\r']).map_or(0, |i| i + 1);
    hooks.contains(name) && text[line_start..a].trim() == "function"
}
