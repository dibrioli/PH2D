//! Gate (b): the canonical sample has ZERO diagnostics — errors, warnings and lints alike — against
//! PH2D's definitions: `BOB_LUAU` itself in non-strict mode, and its typed mirror
//! `fixtures/bob_estrito.luau` in `--!strict`.

use ph2d_code_lsp::Solver;

use crate::support::{bob_luau, diagnostics_of, fixture, ph2d_definitions, render, start};

fn assert_clean(name: &str, text: &str) {
    let mut c = start(&ph2d_definitions(), Solver::Old);
    let diags = diagnostics_of(&mut c, name, text);
    assert!(
        diags.is_empty(),
        "{name}: {} diagnostic(s):{}",
        diags.len(),
        render(&diags)
    );
    c.shutdown().expect("shutdown");
}

fn bob_estrito() -> String {
    std::fs::read_to_string(fixture("bob_estrito.luau")).expect("fixture")
}

#[test]
fn bob_is_clean_in_non_strict_mode() {
    assert_clean("bob.luau", &bob_luau());
}

#[test]
fn bob_is_clean_in_strict_mode() {
    assert_clean("bob_estrito.luau", &bob_estrito());
}

/// The strict mirror is BOB_LUAU plus annotations only: strip its header and the three types and
/// it must be byte-identical — a change to BOB_LUAU that the mirror misses turns this red.
#[test]
fn the_strict_mirror_is_bob_plus_types_only() {
    let mirror = bob_estrito();
    let body = &mirror[mirror.find("-- bob.luau").expect("BOB_LUAU's first line")..];
    let mut untyped = body.to_owned();
    for (typed, plain) in [
        ("function init(self: ScriptSelf)\n", "function init(self)\n"),
        (
            "function update(self: ScriptSelf, dt: number)\n",
            "function update(self, dt)\n",
        ),
        ("  local s: number\n", "  local s\n"),
    ] {
        assert_eq!(untyped.matches(typed).count(), 1, "{typed:?}");
        untyped = untyped.replacen(typed, plain, 1);
    }
    assert_eq!(
        untyped,
        bob_luau(),
        "fixtures/bob_estrito.luau drifted from BOB_LUAU"
    );
}
