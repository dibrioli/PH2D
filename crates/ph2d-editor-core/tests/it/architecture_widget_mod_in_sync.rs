//! Wave 10 / Etapa 4.3 — staleness gate for `ph2d-widget-sync`.
//!
//! Asserts that the `mod foo;` block in `widget/mod.rs` matches a
//! fresh scan of `widget/{*.rs, subdirs/}` — i.e.
//! `cargo run -p ph2d-widget-sync` was rerun after a widget file/dir
//! was added or removed.
//!
//! Only the `mod` block is gated. The `pub use ...` re-exports stay
//! hand-written (per-widget export shapes vary).

use ph2d_widget_sync::{
    MOD_BEGIN, MOD_END, render_mod_lines, scan_widget_dirs, scan_widget_files, splice_lines,
};
use std::path::PathBuf;

fn widget_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src/widget")
}

#[test]
fn widget_mod_block_in_sync_with_folder() {
    let dir = widget_dir();
    let files = scan_widget_files(&dir);
    let dirs = scan_widget_dirs(&dir);
    let mod_path = dir.join("mod.rs");
    let on_disk = std::fs::read_to_string(&mod_path).expect("widget/mod.rs readable");
    let rendered = splice_lines(
        &on_disk,
        MOD_BEGIN,
        MOD_END,
        &render_mod_lines(&files, &dirs),
    );
    assert_eq!(
        on_disk, rendered,
        "widget/mod.rs `mod` declaration block is stale.\n\
         Run `cargo run -p ph2d-widget-sync` and commit the regenerated file."
    );
}
