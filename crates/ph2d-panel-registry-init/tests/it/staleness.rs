//! Wave 10 / Etapa 4.1 — staleness gate for `ph2d-panel-sync`.
//!
//! Fails CI if `cargo run -p ph2d-panel-sync` was not re-run after a
//! panel crate was added or removed (mirror of `ph2d-tool-registry-init`'s
//! staleness pattern from ADR-0040).
//!
//! Three sub-checks:
//!   1. `build_typed_registry` body in `src/lib.rs` matches the rendered
//!      `#[cfg(feature)] reg.push(...)` block from a fresh scan.
//!   2. `[dependencies]` block in `Cargo.toml` matches the rendered deps.
//!   3. `[features]` block in `Cargo.toml` matches the rendered feature
//!      toggles.

use ph2d_panel_sync::{
    TOML_DEPS_BEGIN, TOML_DEPS_END, TOML_FEATURES_BEGIN, TOML_FEATURES_END, crate_ident_for_test,
    extract_registered_panels, panel_struct_name, render_cargo_dep_lines,
    render_cargo_feature_lines, scan_panel_crates, splice_lines,
};
use std::path::PathBuf;

fn crates_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .canonicalize()
        .expect("crates dir resolves")
}

fn render_with_current(path: &PathBuf, begin: &str, end: &str, body: &[String]) -> String {
    let content =
        std::fs::read_to_string(path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
    splice_lines(&content, begin, end, body)
}

#[test]
fn build_typed_registry_in_sync_with_folder() {
    // Wave 10 / Etapa 4 audit [C-1] fix: gate compares SEMANTICALLY
    // (extracted (crate_ident, struct_name) pairs) instead of byte-equal,
    // because rustfmt reformats long ErasedPanel::new::<...> calls to
    // multi-line and the codegen emits single-line. Semantic comparison
    // is whitespace/formatting-tolerant and catches the real failure
    // mode: panel crate added without re-running the sync.
    let crates = crates_dir();
    let panel_crates = scan_panel_crates(&crates);
    let init_lib = crates.join("ph2d-panel-registry-init/src/lib.rs");
    let on_disk = std::fs::read_to_string(&init_lib).expect("lib.rs readable");

    // What's currently in the file:
    let on_disk_pairs = extract_registered_panels(&on_disk);

    // What a fresh scan + render would produce:
    let expected_pairs: Vec<(String, String)> = panel_crates
        .iter()
        .map(|c| {
            let ident = crate_ident_for_test(c);
            let struct_name = panel_struct_name(&crates, c)
                .unwrap_or_else(|| panic!("panel crate `{c}` has no exported `*Panel` struct"));
            (ident, struct_name)
        })
        .collect();

    assert_eq!(
        on_disk_pairs, expected_pairs,
        "ph2d-panel-registry-init/src/lib.rs `build_typed_registry` is stale.\n\
         Run `cargo run -p ph2d-panel-sync` and commit the regenerated file.\n\
         \n\
         on-disk:  {on_disk_pairs:?}\n\
         expected: {expected_pairs:?}"
    );
}

#[test]
fn cargo_deps_in_sync_with_folder() {
    let crates = crates_dir();
    let panel_crates = scan_panel_crates(&crates);
    let cargo = crates.join("ph2d-panel-registry-init/Cargo.toml");
    let on_disk = std::fs::read_to_string(&cargo).expect("Cargo.toml readable");
    let rendered = render_with_current(
        &cargo,
        TOML_DEPS_BEGIN,
        TOML_DEPS_END,
        &render_cargo_dep_lines(&panel_crates),
    );
    assert_eq!(
        on_disk, rendered,
        "ph2d-panel-registry-init/Cargo.toml [dependencies] is stale.\n\
         Run `cargo run -p ph2d-panel-sync` and commit the regenerated file."
    );
}

#[test]
fn cargo_features_in_sync_with_folder() {
    let crates = crates_dir();
    let panel_crates = scan_panel_crates(&crates);
    let cargo = crates.join("ph2d-panel-registry-init/Cargo.toml");
    let on_disk = std::fs::read_to_string(&cargo).expect("Cargo.toml readable");
    let rendered = render_with_current(
        &cargo,
        TOML_FEATURES_BEGIN,
        TOML_FEATURES_END,
        &render_cargo_feature_lines(&panel_crates),
    );
    assert_eq!(
        on_disk, rendered,
        "ph2d-panel-registry-init/Cargo.toml [features] is stale.\n\
         Run `cargo run -p ph2d-panel-sync` and commit the regenerated file."
    );
}
