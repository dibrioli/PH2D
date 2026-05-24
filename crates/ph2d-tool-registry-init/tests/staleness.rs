//! Staleness gate (ADR-0040 §2.3): the codegen'd wiring must match the
//! `crates/ph2d-tool-*` folder. If someone adds/removes a tool crate without
//! running `cargo run -p ph2d-tool-sync`, CI fails here with an actionable
//! message — this is what makes "drop a crate" collision-free without trusting
//! a human to also edit a central list correctly. Twin of the node staleness
//! gate (`ph2d-node-registry-init/tests/staleness.rs`).
//!
//! It re-runs the *exact same* render the sync binary uses (shared via the
//! `ph2d-tool-sync` library) and asserts the checked-in files are byte-identical
//! to a fresh regeneration.

use ph2d_tool_sync::{
    ICON_SLUGS_BEGIN, ICON_SLUGS_END, IMAGE_TOOLS_ORDER_BEGIN, IMAGE_TOOLS_ORDER_END, RS_BEGIN,
    RS_END, TOML_BEGIN, TOML_END, TOOLS_BEGIN, TOOLS_END, filter_exposing, render_cargo_dep_lines,
    render_icon_slug_lines, render_image_tools_order_lines, render_register_lines,
    render_register_tools_lines, scan_design_tomls, scan_tool_crates, splice_lines,
};
use std::path::{Path, PathBuf};

fn crates_dir() -> PathBuf {
    // CARGO_MANIFEST_DIR = .../crates/ph2d-tool-registry-init → parent is crates/.
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .canonicalize()
        .expect("crates dir resolves")
}

fn design_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../docs/design/tools")
        .canonicalize()
        .expect("design dir resolves")
}

#[test]
fn register_all_is_in_sync_with_folder() {
    let crates = crates_dir();
    let manifest_crates = filter_exposing(&crates, &scan_tool_crates(&crates), "pub fn register(");
    let path = crates.join("ph2d-tool-registry-init/src/lib.rs");
    let current = std::fs::read_to_string(&path).expect("lib.rs readable");
    let regenerated = splice_lines(
        &current,
        RS_BEGIN,
        RS_END,
        &render_register_lines(&manifest_crates),
    );
    assert_eq!(
        current, regenerated,
        "register_all is STALE vs crates/ph2d-tool-*. Run: cargo run -p ph2d-tool-sync"
    );
}

#[test]
fn register_all_tools_is_in_sync_with_folder() {
    let crates = crates_dir();
    let modal_crates = filter_exposing(&crates, &scan_tool_crates(&crates), "pub fn make(");
    let path = crates.join("ph2d-tool-registry-init/src/lib.rs");
    let current = std::fs::read_to_string(&path).expect("lib.rs readable");
    let regenerated = splice_lines(
        &current,
        TOOLS_BEGIN,
        TOOLS_END,
        &render_register_tools_lines(&modal_crates),
    );
    assert_eq!(
        current, regenerated,
        "register_all_tools is STALE vs crates/ph2d-tool-* (pub fn make). \
         Run: cargo run -p ph2d-tool-sync"
    );
}

#[test]
fn cargo_deps_in_sync_with_folder() {
    let crates = crates_dir();
    let tool_crates = scan_tool_crates(&crates);
    let path = crates.join("ph2d-tool-registry-init/Cargo.toml");
    let current = std::fs::read_to_string(&path).expect("Cargo.toml readable");
    let regenerated = splice_lines(
        &current,
        TOML_BEGIN,
        TOML_END,
        &render_cargo_dep_lines(&tool_crates),
    );
    assert_eq!(
        current, regenerated,
        "registry-init Cargo deps are STALE vs crates/ph2d-tool-*. Run: cargo run -p ph2d-tool-sync"
    );
}

#[test]
fn image_tools_canonical_order_is_in_sync_with_design_tomls() {
    let crates = crates_dir();
    let entries = scan_design_tomls(&design_dir());
    let path = crates.join("ph2d-tool-registry-init/src/lib.rs");
    let current = std::fs::read_to_string(&path).expect("lib.rs readable");
    let regenerated = splice_lines(
        &current,
        IMAGE_TOOLS_ORDER_BEGIN,
        IMAGE_TOOLS_ORDER_END,
        &render_image_tools_order_lines(&entries),
    );
    assert_eq!(
        current, regenerated,
        "image_tools canonical-order list is STALE vs docs/design/tools/*.toml. \
         Run: cargo run -p ph2d-tool-sync"
    );
}

/// Sanity check: every `.toml` under `docs/design/tools/` must parse
/// into a `DesignEntry`. Without this, a future refactor that silently
/// breaks `parse_design_toml` would drop every TOML and the codegen
/// would emit empty marker blocks — the two staleness gates below
/// would still grumble, but with a generic "lists differ" message
/// and no breadcrumb to the underlying parse failure. This gate fails
/// LOUDLY at parse time (`scan_design_tomls` panics with the file
/// path) before the comparison gates run.
#[test]
fn every_design_toml_parses() {
    let dir = design_dir();
    let on_disk: usize = std::fs::read_dir(&dir)
        .expect("read design dir")
        .filter_map(Result::ok)
        .filter(|e| e.path().extension().and_then(|s| s.to_str()) == Some("toml"))
        .count();
    let parsed = scan_design_tomls(&dir).len();
    assert_eq!(
        parsed,
        on_disk,
        "scan_design_tomls processed {parsed} of {on_disk} *.toml files in {} — \
         a malformed TOML would normally cause the codegen to panic with the \
         file path; if you see this assertion fire, the panic path itself \
         changed.",
        dir.display(),
    );
}

#[test]
fn icon_slug_match_arms_in_sync_with_design_tomls() {
    let crates = crates_dir();
    let entries = scan_design_tomls(&design_dir());
    let path = crates.join("ph2d-tool-registry-init/tests/tool_manifest_design_sync.rs");
    let current = std::fs::read_to_string(&path).expect("design_sync test readable");
    let regenerated = splice_lines(
        &current,
        ICON_SLUGS_BEGIN,
        ICON_SLUGS_END,
        &render_icon_slug_lines(&entries),
    );
    assert_eq!(
        current, regenerated,
        "expected_icon_slug match arms are STALE vs docs/design/tools/*.toml. \
         Run: cargo run -p ph2d-tool-sync"
    );
}
