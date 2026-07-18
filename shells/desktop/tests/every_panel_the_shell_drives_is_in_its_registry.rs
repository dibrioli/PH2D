//! **A panel the shell drives must be a panel the shell BUILT.**
//!
//! This gate exists because the Physics world panel shipped unreachable, with
//! every unit gate green (Enio, smoke 2026-07-18: *"não vejo o painel, não abre
//! com w"*).
//!
//! The mechanism is worth stating exactly, because nothing about it looks like
//! a bug while you are reading it:
//!
//! * `ph2d-panel-registry-init` has a `default` feature list, and its own test
//!   (`build_typed_registry_matches_enabled_features`) checks it — **green**.
//! * The shell then declares
//!   `ph2d-panel-registry-init = { …, default-features = false }` and
//!   re-enumerates the panels it wants in **its own** `[features] default`.
//! * So editing the registry crate's `default` list changes **nothing** about
//!   the running app. A panel is in the product only if it is named in BOTH
//!   places, and the two lists live in different files.
//!
//! Everything downstream then works perfectly on a panel that does not exist:
//! the key handler flips `panel_visibility["physics"]`, the z-order walk asks
//! the registry for that id, gets `None`, and paints nothing. No error, no
//! warning, no missing symbol — the feature is simply absent.
//!
//! The gate is written against the *shell's* view (`cfg!(feature = …)` compiled
//! into this test binary, plus the registry the shell's crate graph produces),
//! because that is the only view that can tell the truth here.

/// Panels the shell actively drives from `render_loop` / input, by `Panel::ID`.
///
/// Add a row when you add a bridge. The point is not the list's length — it is
/// that the list is written where the SHELL is compiled, so a feature flag
/// missing from the shell's `default` cannot hide behind another crate's.
const SHELL_DRIVEN_PANELS: &[(&str, &str)] = &[
    // (Panel::ID, the cargo feature that compiles it into the registry)
    ("physics", "panel-physics"),
    ("timeline", "panel-timeline"),
    ("inspector", "panel-inspector"),
    ("hierarchy", "panel-hierarchy"),
    ("vector", "panel-vector"),
    ("flip", "panel-flip"),
];

/// Which of the above this build actually enabled.
fn enabled(feature: &str) -> bool {
    match feature {
        "panel-physics" => cfg!(feature = "panel-physics"),
        "panel-timeline" => cfg!(feature = "panel-timeline"),
        "panel-inspector" => cfg!(feature = "panel-inspector"),
        "panel-hierarchy" => cfg!(feature = "panel-hierarchy"),
        "panel-vector" => cfg!(feature = "panel-vector"),
        "panel-flip" => cfg!(feature = "panel-flip"),
        other => panic!("unknown feature in SHELL_DRIVEN_PANELS: {other}"),
    }
}

/// **The default build ships every panel the shell drives.**
///
/// Mutation that must bleed: removing `"panel-physics"` from the shell's
/// `[features] default` (which is exactly the state the first smoke ran in).
#[test]
fn the_default_build_enables_every_panel_the_shell_drives() {
    let missing: Vec<&str> = SHELL_DRIVEN_PANELS
        .iter()
        .filter(|(_, feature)| !enabled(feature))
        .map(|(id, _)| *id)
        .collect();
    assert!(
        missing.is_empty(),
        "the shell drives these panels but its DEFAULT build does not compile \
         them in: {missing:?}\n\
         The panel will be invisible in the running app while every unit gate \
         stays green — the key handler toggles a visibility flag, the z-order \
         walk finds no panel for the id, and nothing is painted.\n\
         Fix: add the feature to `shells/desktop/Cargo.toml` `[features] default` \
         AND give it a `panel-x = [\"ph2d-panel-registry-init/panel-x\"]` line. \
         Editing the registry crate's own `default` list does NOT reach the shell \
         (it is declared `default-features = false`)."
    );
}

/// **And the registry the shell's crate graph builds actually contains them.**
///
/// The feature being on is necessary, not sufficient: the push into the
/// registry is codegen (`cargo run -p ph2d-panel-sync`), so a crate whose
/// `pub struct <Name>Panel` was renamed, or whose generated block went stale,
/// is enabled and still absent. This asks the built registry itself.
#[test]
fn the_built_registry_contains_every_panel_the_shell_drives() {
    let registry = ph2d_panel_registry_init::build_typed_registry();
    let ids: Vec<&str> = registry.panels().iter().map(|p| p.manifest.id).collect();

    let missing: Vec<&str> = SHELL_DRIVEN_PANELS
        .iter()
        .filter(|(_, feature)| enabled(feature))
        .map(|(id, _)| *id)
        .filter(|id| !ids.contains(id))
        .collect();
    assert!(
        missing.is_empty(),
        "these panels are ENABLED but absent from the registry the shell builds: \
         {missing:?}\nregistry has: {ids:?}\n\
         The feature flag is on, so the crate compiled — but the push is codegen: \
         re-run `cargo run -p ph2d-panel-sync` and check the crate still exports \
         `pub struct <Name>Panel` at its lib root."
    );
}
