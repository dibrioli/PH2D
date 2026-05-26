//! Staleness gate (ADR-0054 §2.2): the codegen'd wiring must match the
//! `crates/ph2d-imageio-*` folder. If someone adds/removes a format
//! crate without running `cargo run -p ph2d-imageio-sync`, CI fails here
//! with an actionable message — this is what makes "drop a crate"
//! collision-free without trusting a human to also edit a central list
//! correctly. Twin of `ph2d-tool-registry-init/tests/staleness.rs` and
//! `ph2d-node-registry-init/tests/staleness.rs`.
//!
//! Re-runs the *exact same* render the sync binary uses (shared via the
//! `ph2d-imageio-sync` library) and asserts the marker block in the
//! checked-in file matches a fresh regeneration. Asserts on the block
//! only, not the whole file, so the failure diff stays focused on the
//! stale lines.

use ph2d_imageio_sync::{
    EXPORTERS_BEGIN, EXPORTERS_END, IMPORTERS_BEGIN, IMPORTERS_END, TOML_BEGIN, TOML_END,
    filter_exposing, render_cargo_dep_lines, render_register_exporter_lines,
    render_register_importer_lines, scan_imageio_crates,
};
use std::path::{Path, PathBuf};

fn crates_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .canonicalize()
        .expect("crates dir resolves")
}

/// Extract the lines strictly *between* `begin` and `end` marker lines
/// (markers excluded). Marker match is trim-aware. Panics if either
/// marker is missing — that's a setup bug, not a drift the user can fix
/// by re-running the sync.
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
fn register_all_importers_is_in_sync_with_folder() {
    let crates = crates_dir();
    let importer_crates = filter_exposing(
        &crates,
        &scan_imageio_crates(&crates),
        "pub fn register_importer(",
    );
    let path = crates.join("ph2d-imageio-registry-init/src/lib.rs");
    let current = std::fs::read_to_string(&path).expect("lib.rs readable");
    let actual = extract_block(&current, IMPORTERS_BEGIN, IMPORTERS_END);
    let expected = render_register_importer_lines(&importer_crates);
    assert_eq!(
        actual, expected,
        "register_all_importers is STALE vs crates/ph2d-imageio-* (pub fn \
         register_importer). Run: cargo run -p ph2d-imageio-sync"
    );
}

#[test]
fn register_all_exporters_is_in_sync_with_folder() {
    let crates = crates_dir();
    let exporter_crates = filter_exposing(
        &crates,
        &scan_imageio_crates(&crates),
        "pub fn register_exporter(",
    );
    let path = crates.join("ph2d-imageio-registry-init/src/lib.rs");
    let current = std::fs::read_to_string(&path).expect("lib.rs readable");
    let actual = extract_block(&current, EXPORTERS_BEGIN, EXPORTERS_END);
    let expected = render_register_exporter_lines(&exporter_crates);
    assert_eq!(
        actual, expected,
        "register_all_exporters is STALE vs crates/ph2d-imageio-* (pub fn \
         register_exporter). Run: cargo run -p ph2d-imageio-sync"
    );
}

#[test]
fn cargo_deps_in_sync_with_folder() {
    let crates = crates_dir();
    let format_crates = scan_imageio_crates(&crates);
    let path = crates.join("ph2d-imageio-registry-init/Cargo.toml");
    let current = std::fs::read_to_string(&path).expect("Cargo.toml readable");
    let actual = extract_block(&current, TOML_BEGIN, TOML_END);
    let expected = render_cargo_dep_lines(&format_crates);
    assert_eq!(
        actual, expected,
        "registry-init Cargo deps are STALE vs crates/ph2d-imageio-*. \
         Run: cargo run -p ph2d-imageio-sync"
    );
}
