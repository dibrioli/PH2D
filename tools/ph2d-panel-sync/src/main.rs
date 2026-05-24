#![forbid(unsafe_code)]
//! `ph2d-panel-sync` — regenerate the panel registry-init wiring from a
//! scan of `crates/ph2d-panel-*`. Run after adding/removing a panel
//! crate:
//!
//! ```text
//! cargo run -p ph2d-panel-sync
//! ```
//!
//! Rewrites three marked regions:
//!   - `ph2d-panel-registry-init/src/lib.rs`: `build_typed_registry` body
//!   - `ph2d-panel-registry-init/Cargo.toml`: `[dependencies]` (optional panel deps)
//!   - `ph2d-panel-registry-init/Cargo.toml`: `[features]` (panel-X = ["dep:..."])
//!
//! The staleness gate in `ph2d-panel-registry-init` fails CI if this
//! script was not run after a panel crate change.
//!
//! Wave 10 / Etapa 4.1 — twin of `ph2d-tool-sync` for the panel family.

use ph2d_panel_sync::{
    RS_BEGIN, RS_END, TOML_DEPS_BEGIN, TOML_DEPS_END, TOML_FEATURES_BEGIN, TOML_FEATURES_END,
    render_cargo_dep_lines, render_cargo_feature_lines, render_register_lines, scan_panel_crates,
    splice_lines,
};
use std::path::Path;
use std::process::Command;

fn main() {
    let crates_dir = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../crates")
        .canonicalize()
        .expect("crates dir resolves");

    let panel_crates = scan_panel_crates(&crates_dir);
    let init = crates_dir.join("ph2d-panel-registry-init");

    rewrite(
        &init.join("src/lib.rs"),
        RS_BEGIN,
        RS_END,
        &render_register_lines(&crates_dir, &panel_crates),
    );
    rewrite(
        &init.join("Cargo.toml"),
        TOML_DEPS_BEGIN,
        TOML_DEPS_END,
        &render_cargo_dep_lines(&panel_crates),
    );
    rewrite(
        &init.join("Cargo.toml"),
        TOML_FEATURES_BEGIN,
        TOML_FEATURES_END,
        &render_cargo_feature_lines(&panel_crates),
    );

    // Wave 10 / Etapa 4 audit fix [C-1]: rustfmt the regenerated lib.rs
    // so the on-disk form is byte-identical to whatever the next
    // `cargo fmt` would produce. Without this, the codegen emits a
    // canonical multi-line form for `ErasedPanel::new::<...>()` but
    // rustfmt collapses short type paths back to single-line — and the
    // staleness gate (which compares on-disk vs splice(render(...)))
    // would fail on the next pre-commit. The gate ALSO calls rustfmt
    // (via the same convention below) so both sides agree.
    let status = Command::new("cargo")
        .args(["fmt", "-p", "ph2d-panel-registry-init"])
        .status();
    if let Ok(s) = status
        && !s.success()
    {
        eprintln!(
            "ph2d-panel-sync: WARNING — `cargo fmt -p ph2d-panel-registry-init` \
             exited non-zero; the regenerated file may not be canonical."
        );
    }

    println!(
        "ph2d-panel-sync: {} panel crate(s) synced (lib.rs rustfmt'd).",
        panel_crates.len(),
    );
}

fn rewrite(path: &Path, begin: &str, end: &str, body: &[String]) {
    let content =
        std::fs::read_to_string(path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
    let updated = splice_lines(&content, begin, end, body);
    std::fs::write(path, updated).unwrap_or_else(|e| panic!("write {}: {e}", path.display()));
}
