#![forbid(unsafe_code)]
//! `ph2d-tool-sync` — regenerate the tool registry-init wiring from a scan of
//! `crates/ph2d-tool-*`. Run after adding/removing a tool crate:
//!
//! ```text
//! cargo run -p ph2d-tool-sync
//! ```
//!
//! Rewrites the marked regions of `ph2d-tool-registry-init`'s `src/lib.rs`
//! (the `register_all` body) and `Cargo.toml` (the `[dependencies]`). The
//! staleness gate in that crate fails CI if this was not run. Twin of
//! `ph2d-node-sync` (ADR-0040).

use ph2d_tool_sync::{
    RS_BEGIN, RS_END, TOML_BEGIN, TOML_END, render_cargo_dep_lines, render_register_lines,
    scan_tool_crates, splice_lines,
};
use std::path::Path;

fn main() {
    let crates_dir = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../crates")
        .canonicalize()
        .expect("crates dir resolves");

    let tool_crates = scan_tool_crates(&crates_dir);
    let init = crates_dir.join("ph2d-tool-registry-init");

    rewrite(
        &init.join("src/lib.rs"),
        RS_BEGIN,
        RS_END,
        &render_register_lines(&tool_crates),
    );
    rewrite(
        &init.join("Cargo.toml"),
        TOML_BEGIN,
        TOML_END,
        &render_cargo_dep_lines(&tool_crates),
    );

    println!(
        "ph2d-tool-sync: wired {} tool crate(s): {:?}",
        tool_crates.len(),
        tool_crates
    );
}

fn rewrite(path: &Path, begin: &str, end: &str, body: &[String]) {
    let content =
        std::fs::read_to_string(path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
    let updated = splice_lines(&content, begin, end, body);
    std::fs::write(path, updated).unwrap_or_else(|e| panic!("write {}: {e}", path.display()));
}
