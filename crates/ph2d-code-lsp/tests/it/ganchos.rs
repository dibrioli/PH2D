//! The engine hooks (`init` · `update` · `on_signal`, the `-- @hook` lines that `ph2d.d.luau` gets from
//! `api_table::HOOKS`) are not reported as unused — and the filter takes NOTHING else: an unused local
//! function, a misspelt hook (`updte`) and a LOCAL function named like a hook still warn.

use std::collections::BTreeSet;

use ph2d_code_lsp::{Solver, hooks_in_definitions};

use crate::support::{
    assert_exactly, diagnostics_of, expected_from_markers, fixture, ph2d_definitions, start,
};

#[test]
fn hooks_are_read_from_the_definitions_file() {
    let defs = std::fs::read_to_string(ph2d_definitions()).expect("ph2d.d.luau");
    let hooks: BTreeSet<String> = hooks_in_definitions(&defs).into_iter().collect();
    for h in ["init", "update", "on_signal"] {
        assert!(
            hooks.contains(h),
            "ph2d.d.luau has no `-- @hook {h}:` line: {hooks:?}"
        );
    }
    let c = start(&ph2d_definitions(), Solver::Old);
    assert_eq!(c.engine_hooks(), &hooks);
    c.shutdown().expect("shutdown");
}

#[test]
fn only_the_unused_warning_of_a_global_hook_is_dropped() {
    let text = std::fs::read_to_string(fixture("ganchos.luau")).expect("fixture");
    let expected = expected_from_markers(&text);
    assert_eq!(expected.len(), 3, "fixture lost its ERR markers");
    let mut c = start(&ph2d_definitions(), Solver::Old);
    let diags = diagnostics_of(&mut c, "ganchos.luau", &text);
    c.shutdown().expect("shutdown");
    assert_exactly(&diags, &expected);
}
