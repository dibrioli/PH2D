//! Staleness gate (ADR-0031, ADR-0032): the codegen'd wiring must match the
//! `crates/ph2d-node-*` folder. If someone adds/removes a node crate without
//! running `cargo run -p ph2d-node-sync`, CI fails here with an actionable
//! message — this is what makes "drop a crate" collision-free without trusting
//! a human to also edit a central list correctly.
//!
//! Asserts on the marker block only (not the whole file), so the failure
//! diff stays focused on the stale lines — same pattern as the twin tool
//! gate (`ph2d-tool-registry-init/tests/staleness.rs`).

use ph2d_node_sync::{
    RS_BEGIN, RS_END, TOML_BEGIN, TOML_END, render_cargo_dep_lines, render_register_lines,
    scan_node_crates,
};
use std::path::{Path, PathBuf};

fn crates_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .canonicalize()
        .expect("crates dir resolves")
}

/// Extract the lines strictly *between* `begin` and `end` marker lines in
/// `content` (markers themselves excluded). Marker match is trim-aware.
fn extract_block(content: &str, begin: &str, end: &str) -> Vec<String> {
    let lines: Vec<&str> = content.lines().collect();
    let bi = lines
        .iter()
        .position(|l| l.trim() == begin.trim())
        .unwrap_or_else(|| panic!("begin marker missing in source: {begin}"));
    let ei = lines
        .iter()
        .position(|l| l.trim() == end.trim())
        .unwrap_or_else(|| panic!("end marker missing in source: {end}"));
    assert!(bi < ei, "begin marker must precede end marker: {begin}");
    lines[bi + 1..ei].iter().map(|s| s.to_string()).collect()
}

#[test]
fn register_all_nodes_is_in_sync_with_folder() {
    let crates = crates_dir();
    let node_crates = scan_node_crates(&crates);
    let path = crates.join("ph2d-node-registry-init/src/lib.rs");
    let current = std::fs::read_to_string(&path).expect("lib.rs readable");
    let actual = extract_block(&current, RS_BEGIN, RS_END);
    let expected = render_register_lines(&node_crates);
    assert_eq!(
        actual, expected,
        "register_all_nodes is STALE vs crates/ph2d-node-*. Run: cargo run -p ph2d-node-sync"
    );
}

#[test]
fn cargo_deps_in_sync_with_folder() {
    let crates = crates_dir();
    let node_crates = scan_node_crates(&crates);
    let path = crates.join("ph2d-node-registry-init/Cargo.toml");
    let current = std::fs::read_to_string(&path).expect("Cargo.toml readable");
    let actual = extract_block(&current, TOML_BEGIN, TOML_END);
    let expected = render_cargo_dep_lines(&node_crates);
    assert_eq!(
        actual, expected,
        "registry-init Cargo deps are STALE vs crates/ph2d-node-*. Run: cargo run -p ph2d-node-sync"
    );
}
