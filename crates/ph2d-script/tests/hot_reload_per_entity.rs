//! Per-entity hot-reload contract test (ADR-0025 M14.2).
//!
//! When `ScriptHost::load_script` rebuilds the Luau VM (because the
//! source bytes changed), the per-entity lateral state must survive.
//! Two paths to verify:
//!
//! 1. State seeded by Rust via `provide_state` is visible to the
//!    fresh VM.
//! 2. State written by a previous script version is visible to the
//!    new script version (after reload).

use ph2d_script::{PodValue, ScriptHost};

#[test]
fn provide_state_survives_load_script_reset() {
    let mut host = ScriptHost::new().unwrap();
    host.provide_state(42, "hp", PodValue::Number(100.0));

    // Initial load.
    let changed = host.load_script("-- v1: empty\n").unwrap();
    assert!(changed);
    let v1: f64 = host
        .runtime()
        .lua()
        .load("return ph2d.state_get(42, 'hp')")
        .eval()
        .unwrap();
    assert_eq!(v1, 100.0);

    // Source bytes change → VM rebuilds.
    let changed = host.load_script("-- v2: different\n").unwrap();
    assert!(changed);
    assert!(host.reset_count() >= 2, "expected at least two resets");

    // State that the host seeded is still there.
    let v2: f64 = host
        .runtime()
        .lua()
        .load("return ph2d.state_get(42, 'hp')")
        .eval()
        .unwrap();
    assert_eq!(v2, 100.0, "lateral state lost across hot reload");
}

#[test]
fn lua_written_state_survives_reload() {
    let mut host = ScriptHost::new().unwrap();
    // v1 script writes a counter at init time.
    host.load_script("ph2d.state_set(7, 'counter', 5)").unwrap();
    let v1: f64 = host
        .runtime()
        .lua()
        .load("return ph2d.state_get(7, 'counter')")
        .eval()
        .unwrap();
    assert_eq!(v1, 5.0);

    // v2 script doesn't touch the field — but must see it.
    host.load_script("-- v2: no-op").unwrap();
    let v2: f64 = host
        .runtime()
        .lua()
        .load("return ph2d.state_get(7, 'counter')")
        .eval()
        .unwrap();
    assert_eq!(v2, 5.0, "Lua-written lateral state lost across hot reload");
}

#[test]
fn reload_with_identical_source_is_noop() {
    let mut host = ScriptHost::new().unwrap();
    let changed = host.load_script("-- same").unwrap();
    assert!(changed, "first load always reports change");
    let initial_resets = host.reset_count();
    let changed = host.load_script("-- same").unwrap();
    assert!(!changed, "identical source must skip rebuild");
    assert_eq!(host.reset_count(), initial_resets);
}
