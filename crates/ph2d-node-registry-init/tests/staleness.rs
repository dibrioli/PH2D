//! Staleness gate (ADR-0031, ADR-0032): the codegen'd wiring must match the
//! `crates/ph2d-node-*` folder. If someone adds/removes a node crate without
//! running `cargo run -p ph2d-node-sync`, CI fails here with an actionable
//! message — this is what makes "drop a crate" collision-free without trusting
//! a human to also edit a central list correctly.
//!
//! It re-runs the *exact same* render the sync binary uses (shared via the
//! `ph2d-node-sync` library) and asserts the checked-in files are byte-identical
//! to a fresh regeneration.

use ph2d_node_sync::{
    RS_BEGIN, RS_END, TOML_BEGIN, TOML_END, render_cargo_dep_lines, render_register_lines,
    scan_node_crates, splice_lines,
};
use std::path::{Path, PathBuf};

fn crates_dir() -> PathBuf {
    // CARGO_MANIFEST_DIR = .../crates/ph2d-node-registry-init → parent is crates/.
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .canonicalize()
        .expect("crates dir resolves")
}

#[test]
fn register_all_nodes_is_in_sync_with_folder() {
    let crates = crates_dir();
    let node_crates = scan_node_crates(&crates);
    let path = crates.join("ph2d-node-registry-init/src/lib.rs");
    let current = std::fs::read_to_string(&path).expect("lib.rs readable");
    let regenerated = splice_lines(
        &current,
        RS_BEGIN,
        RS_END,
        &render_register_lines(&node_crates),
    );
    assert_eq!(
        current, regenerated,
        "register_all_nodes is STALE vs crates/ph2d-node-*. Run: cargo run -p ph2d-node-sync"
    );
}

#[test]
fn cargo_deps_in_sync_with_folder() {
    let crates = crates_dir();
    let node_crates = scan_node_crates(&crates);
    let path = crates.join("ph2d-node-registry-init/Cargo.toml");
    let current = std::fs::read_to_string(&path).expect("Cargo.toml readable");
    let regenerated = splice_lines(
        &current,
        TOML_BEGIN,
        TOML_END,
        &render_cargo_dep_lines(&node_crates),
    );
    assert_eq!(
        current, regenerated,
        "registry-init Cargo deps are STALE vs crates/ph2d-node-*. Run: cargo run -p ph2d-node-sync"
    );
}
