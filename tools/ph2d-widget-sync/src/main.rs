#![forbid(unsafe_code)]
//! `ph2d-widget-sync` — regenerate the widget `mod` declaration block
//! from a scan of `crates/ph2d-editor-core/src/widget/{*.rs,subdirs/}`.
//!
//! ```text
//! cargo run -p ph2d-widget-sync
//! ```

use ph2d_widget_sync::{
    MOD_BEGIN, MOD_END, render_mod_lines, scan_widget_dirs, scan_widget_files, splice_lines,
};
use std::path::Path;

fn main() {
    let widget_dir = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../crates/ph2d-editor-core/src/widget")
        .canonicalize()
        .expect("widget dir resolves");

    let files = scan_widget_files(&widget_dir);
    let dirs = scan_widget_dirs(&widget_dir);
    let mod_file = widget_dir.join("mod.rs");

    let content = std::fs::read_to_string(&mod_file).expect("widget/mod.rs readable");
    let updated = splice_lines(
        &content,
        MOD_BEGIN,
        MOD_END,
        &render_mod_lines(&files, &dirs),
    );
    std::fs::write(&mod_file, updated).expect("widget/mod.rs writable");

    println!(
        "ph2d-widget-sync: {} file widget(s) + {} sub-dir widget(s) synced.",
        files.len(),
        dirs.len(),
    );
}
