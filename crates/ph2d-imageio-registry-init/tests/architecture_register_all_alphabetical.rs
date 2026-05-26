//! Arch-gate: `register_all_importers` + `register_all_exporters` +
//! Cargo `[dependencies]` codegen blocks must be alphabetically sorted.
//! Audit C-H2 (2026-05-26) — port of
//! `ph2d-tool-registry-init/tests/architecture_register_all_alphabetical.rs`
//! for symmetric multi-agent collision defence.
//!
//! Three surfaces every new format crate touches via codegen:
//! `register_all_importers` body, `register_all_exporters` body, and
//! the `[dependencies]` block in this crate's `Cargo.toml`. Forcing
//! alphabetical order means concurrent inserts from parallel agents
//! land in distinct positions of the file with deterministic merge
//! resolution: `git merge` produces a conflict if and only if the
//! SAME letter is inserted twice, which is fast to resolve by hand.
//! Random-order inserts produce semantic conflicts the merge driver
//! can't even diagnose.
//!
//! All three regions are codegen'd by `ph2d-imageio-sync`, so a fresh
//! sort comes free from `cargo run -p ph2d-imageio-sync`.

use std::path::PathBuf;

fn crate_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

#[test]
fn register_all_importers_body_is_alphabetical() {
    let src = std::fs::read_to_string(crate_root().join("src/lib.rs"))
        .expect("src/lib.rs must be readable");
    let crates: Vec<&str> = src
        .lines()
        .map(str::trim)
        .filter_map(|l| {
            // Match `ph2d_imageio_<slug>::register_importer(reg);` only
            // (skip commented placeholder lines starting with `//`).
            let stripped = l.strip_prefix("ph2d_imageio_")?;
            let slug_end = stripped.find("::register_importer(reg);")?;
            Some(&stripped[..slug_end])
        })
        .collect();
    // Empty is valid (W0 state has only `png`; later fan-out batches
    // add more). The sortedness check still has to be defensive.
    let mut sorted = crates.clone();
    sorted.sort_unstable();
    assert_eq!(
        crates, sorted,
        "register_all_importers() calls must be in alphabetical order. \
         actual={crates:?} sorted={sorted:?}. Run: cargo run -p \
         ph2d-imageio-sync"
    );
}

#[test]
fn register_all_exporters_body_is_alphabetical() {
    let src = std::fs::read_to_string(crate_root().join("src/lib.rs"))
        .expect("src/lib.rs must be readable");
    let crates: Vec<&str> = src
        .lines()
        .map(str::trim)
        .filter_map(|l| {
            let stripped = l.strip_prefix("ph2d_imageio_")?;
            let slug_end = stripped.find("::register_exporter(reg);")?;
            Some(&stripped[..slug_end])
        })
        .collect();
    let mut sorted = crates.clone();
    sorted.sort_unstable();
    assert_eq!(
        crates, sorted,
        "register_all_exporters() calls must be in alphabetical order. \
         actual={crates:?} sorted={sorted:?}. Run: cargo run -p \
         ph2d-imageio-sync"
    );
}

#[test]
fn cargo_dependencies_block_is_alphabetical() {
    let src = std::fs::read_to_string(crate_root().join("Cargo.toml"))
        .expect("Cargo.toml must be readable");
    // Extract crate names between the sync markers.
    let begin_marker = "# <ph2d-imageio-sync:begin>";
    let end_marker = "# <ph2d-imageio-sync:end>";
    let bi = src
        .find(begin_marker)
        .expect("begin marker present in Cargo.toml");
    let ei = src.find(end_marker).expect("end marker present");
    assert!(bi < ei, "markers in wrong order");
    let region = &src[bi..ei];
    let crates: Vec<&str> = region
        .lines()
        .filter_map(|l| {
            let l = l.trim();
            let name_end = l.find(" = ")?;
            let name = &l[..name_end];
            if name.starts_with("ph2d-imageio-") {
                Some(name)
            } else {
                None
            }
        })
        .collect();
    let mut sorted = crates.clone();
    sorted.sort_unstable();
    assert_eq!(
        crates, sorted,
        "[dependencies] block (between sync markers) must be in \
         alphabetical order. actual={crates:?} sorted={sorted:?}. \
         Run: cargo run -p ph2d-imageio-sync"
    );
}
