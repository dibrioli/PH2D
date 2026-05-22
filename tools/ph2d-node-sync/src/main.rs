#![forbid(unsafe_code)]
//! `ph2d-node-sync` — regenerate the node registry-init wiring from a scan of
//! `crates/ph2d-node-*`. Run after adding/removing a node crate:
//!
//! ```text
//! cargo run -p ph2d-node-sync
//! ```
//!
//! Rewrites the marked regions of `ph2d-node-registry-init`'s `src/lib.rs`
//! (the `register_all_nodes` body) and `Cargo.toml` (the `[dependencies]`).
//! The staleness gate in that crate fails CI if this was not run.

use ph2d_node_sync::{
    RS_BEGIN, RS_END, TOML_BEGIN, TOML_END, render_cargo_dep_lines, render_register_lines,
    scan_node_crates, splice_lines,
};
use std::path::Path;

fn main() {
    let crates_dir = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../crates")
        .canonicalize()
        .expect("crates dir resolves");

    let node_crates = scan_node_crates(&crates_dir);
    let init = crates_dir.join("ph2d-node-registry-init");

    rewrite(
        &init.join("src/lib.rs"),
        RS_BEGIN,
        RS_END,
        &render_register_lines(&node_crates),
    );
    rewrite(
        &init.join("Cargo.toml"),
        TOML_BEGIN,
        TOML_END,
        &render_cargo_dep_lines(&node_crates),
    );

    println!(
        "ph2d-node-sync: wired {} node crate(s): {:?}",
        node_crates.len(),
        node_crates
    );
}

fn rewrite(path: &Path, begin: &str, end: &str, body: &[String]) {
    let content =
        std::fs::read_to_string(path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
    let updated = splice_lines(&content, begin, end, body);
    std::fs::write(path, updated).unwrap_or_else(|e| panic!("write {}: {e}", path.display()));
}
