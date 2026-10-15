//! Gate (b): the canonical sample `bob.luau` (the components smoke) has ZERO diagnostics — errors,
//! warnings and lints alike — against PH2D's definitions, in non-strict and in `--!strict`.

use ph2d_code_lsp::Solver;

use crate::support::{bob_luau, diagnostics_of, ph2d_definitions, render, start};

fn assert_clean(name: &str, text: &str) {
    let mut c = start(&ph2d_definitions(), Solver::Old);
    let diags = diagnostics_of(&mut c, name, text);
    assert!(diags.is_empty(), "{name}: {} diagnostic(s):{}", diags.len(), render(&diags));
    c.shutdown().expect("shutdown");
}

#[test]
fn bob_is_clean_in_non_strict_mode() {
    assert_clean("bob.luau", &bob_luau());
}

#[test]
fn bob_is_clean_in_strict_mode() {
    assert_clean("bob_strict.luau", &format!("--!strict\n{}", bob_luau()));
}
