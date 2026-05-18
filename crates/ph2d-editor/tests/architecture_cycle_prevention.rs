//! Wave 8 Phase 1.E — architecture invariants for the panel
//! ecosystem dependency graph.
//!
//! The Wave-7 audit's P3 finding flagged that nothing guarded
//! against a future regression where:
//!
//! 1. `ph2d-editor-core` accidentally gains a dep on a `ph2d-panel-*`
//!    crate (or on `ph2d-editor`), inverting the foundation layer.
//! 2. A panel crate keeps depending on `ph2d-editor` post Wave 8
//!    Phase 2 (Stage 4 migration), defeating the panel-as-crate
//!    isolation promise.
//!
//! This test reads each crate's `Cargo.toml` and asserts the
//! invariants. Dep-free — parses TOML as text since `serde` /
//! `toml` aren't workspace dev-deps and the `[dependencies]` shape
//! we care about is single-line-per-entry.
//!
//! See `docs/architecture/decisions/0028-wave-2-codegen-design-canonical.md`
//! §Wave 8 for the rationale + the two-phase rollout of these
//! invariants.

use std::path::{Path, PathBuf};

fn workspace_root() -> PathBuf {
    // CARGO_MANIFEST_DIR points at this crate (`crates/ph2d-editor`);
    // the workspace root is two parents up.
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("crates/")
        .parent()
        .expect("workspace root")
        .to_path_buf()
}

/// Parse the `[dependencies]` block of a Cargo.toml and return the
/// dep names (left of `=`). Strips comments, blank lines, and
/// other section headers.
fn parse_deps(toml_path: &Path) -> Vec<String> {
    let text = std::fs::read_to_string(toml_path)
        .unwrap_or_else(|e| panic!("read {}: {e}", toml_path.display()));
    let mut out = Vec::new();
    let mut in_deps = false;
    for raw in text.lines() {
        let line = raw.trim();
        if line.starts_with('#') || line.is_empty() {
            continue;
        }
        if line.starts_with('[') {
            in_deps = line == "[dependencies]";
            continue;
        }
        if !in_deps {
            continue;
        }
        if let Some(eq) = line.find('=') {
            let name = line[..eq].trim();
            if !name.is_empty() {
                out.push(name.to_string());
            }
        }
    }
    out
}

/// All `ph2d-panel-*` crate paths under `crates/`. Hardcoded to
/// the four known panels; if a new panel crate joins, add it here.
fn panel_crate_tomls() -> Vec<PathBuf> {
    let root = workspace_root();
    [
        "ph2d-panel-widget-gallery",
        "ph2d-panel-hierarchy",
        "ph2d-panel-inspector",
        "ph2d-panel-grid-snap",
    ]
    .into_iter()
    .map(|name| root.join("crates").join(name).join("Cargo.toml"))
    .collect()
}

#[test]
fn editor_core_has_no_panel_or_editor_deps() {
    let toml = workspace_root()
        .join("crates")
        .join("ph2d-editor-core")
        .join("Cargo.toml");
    let deps = parse_deps(&toml);
    for dep in &deps {
        assert!(
            !dep.starts_with("ph2d-panel-"),
            "INVARIANT VIOLATED — `ph2d-editor-core` depends on `{dep}`. \
             editor-core is the foundation layer; panel crates must \
             depend on it, not the other way around. Move the shared \
             code DOWN into editor-core (or another shared crate) \
             instead of pulling a panel UP."
        );
        assert_ne!(
            dep, "ph2d-editor",
            "INVARIANT VIOLATED — `ph2d-editor-core` depends on `ph2d-editor`. \
             That's a cycle: editor-core is the foundation, ph2d-editor \
             builds on it."
        );
    }
}

/// Wave 8 Phase 2 invariant — every `ph2d-panel-*` crate depends on
/// `ph2d-editor-core` but NOT on `ph2d-editor`. This is the
/// definition of "physical panel-as-crate isolation"; once it
/// passes, a 3rd-party panel can `crates.io` itself and live
/// outside the workspace without depending on `ph2d-editor`.
///
/// Marked `#[ignore]` for now because Wave 8 Phase 1 still has 3
/// alias panel crates (inspector, hierarchy, grid-snap) that
/// re-export `ph2d_editor::*::PANEL_MANIFEST`. Phase 2 (Stage 4)
/// physically migrates each panel body into its crate; the
/// `#[ignore]` lifts in that commit.
#[test]
#[ignore = "Wave 8 Phase 2 (Stage 4 panel-body migration) unblocks this"]
fn panel_crates_depend_only_on_editor_core() {
    let mut violations = Vec::new();
    for toml in panel_crate_tomls() {
        let panel = toml
            .parent()
            .and_then(|p| p.file_name())
            .map(|s| s.to_string_lossy().into_owned())
            .unwrap_or_else(|| toml.display().to_string());
        let deps = parse_deps(&toml);
        let has_core = deps.iter().any(|d| d == "ph2d-editor-core");
        let has_editor = deps.iter().any(|d| d == "ph2d-editor");
        if !has_core {
            violations.push(format!(
                "{panel} must declare `ph2d-editor-core` in [dependencies]"
            ));
        }
        if has_editor {
            violations.push(format!(
                "{panel} depends on `ph2d-editor` — Stage 4 invariant requires \
                 panel crates to consume `ph2d-editor-core` only"
            ));
        }
    }
    assert!(
        violations.is_empty(),
        "Panel-crate dependency invariant violated:\n  - {}",
        violations.join("\n  - ")
    );
}

#[test]
fn parse_deps_smoke() {
    // Self-check: editor-core has the deps we expect (sanity that
    // `parse_deps` is reading the file, not silently returning
    // empty due to a TOML shape we didn't anticipate).
    let toml = workspace_root()
        .join("crates")
        .join("ph2d-editor-core")
        .join("Cargo.toml");
    let deps = parse_deps(&toml);
    assert!(
        deps.iter().any(|d| d == "ph2d-tokens"),
        "expected `ph2d-tokens` in editor-core deps, got: {deps:?}"
    );
}
