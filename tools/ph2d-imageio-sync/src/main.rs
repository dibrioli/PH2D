#![forbid(unsafe_code)]
//! `ph2d-imageio-sync` — regenerate the imageio registry-init wiring from
//! a scan of `crates/ph2d-imageio-*`. Run after adding/removing an
//! imageio format crate:
//!
//! ```text
//! cargo run -p ph2d-imageio-sync
//! ```
//!
//! Rewrites three marked regions: in `ph2d-imageio-registry-init`'s
//! `src/lib.rs` the `register_all_importers` body (crates with
//! `pub fn register_importer`) and the `register_all_exporters` body
//! (crates with `pub fn register_exporter`), and in its `Cargo.toml` the
//! `[dependencies]` (all imageio format crates). The staleness gate in
//! that crate fails CI if this was not run. Twin of `ph2d-tool-sync`
//! (ADR-0040) and `ph2d-node-sync` (ADR-0039).

use ph2d_imageio_sync::{
    EXPORTERS_BEGIN, EXPORTERS_END, IMPORTERS_BEGIN, IMPORTERS_END, TOML_BEGIN, TOML_END,
    filter_exposing, render_cargo_dep_lines, render_register_exporter_lines,
    render_register_importer_lines, scan_imageio_crates, splice_lines,
};
use std::path::Path;

fn main() {
    let crates_dir = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../crates")
        .canonicalize()
        .expect("crates dir resolves");

    let format_crates = scan_imageio_crates(&crates_dir);
    // Split by capability. Needles include the opening paren so the scan
    // only matches the exact target functions — `pub fn
    // register_importer(reg: &mut ImporterRegistry)` and `pub fn
    // register_exporter(reg: &mut ExporterRegistry)` — never a future
    // helper like `pub fn register_importer_for_test`.
    let importer_crates = filter_exposing(&crates_dir, &format_crates, "pub fn register_importer(");
    let exporter_crates = filter_exposing(&crates_dir, &format_crates, "pub fn register_exporter(");
    let init = crates_dir.join("ph2d-imageio-registry-init");

    rewrite(
        &init.join("src/lib.rs"),
        IMPORTERS_BEGIN,
        IMPORTERS_END,
        &render_register_importer_lines(&importer_crates),
    );
    rewrite(
        &init.join("src/lib.rs"),
        EXPORTERS_BEGIN,
        EXPORTERS_END,
        &render_register_exporter_lines(&exporter_crates),
    );
    rewrite(
        &init.join("Cargo.toml"),
        TOML_BEGIN,
        TOML_END,
        &render_cargo_dep_lines(&format_crates),
    );

    println!(
        "ph2d-imageio-sync: {} format crate(s); {} importer, {} exporter.",
        format_crates.len(),
        importer_crates.len(),
        exporter_crates.len(),
    );
}

fn rewrite(path: &Path, begin: &str, end: &str, body: &[String]) {
    let content =
        std::fs::read_to_string(path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
    let updated = splice_lines(&content, begin, end, body);
    std::fs::write(path, updated).unwrap_or_else(|e| panic!("write {}: {e}", path.display()));
}
