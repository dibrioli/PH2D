#![forbid(unsafe_code)]
//! `ph2d-chrome-sync` — regenerate the chrome `mod` declaration block
//! from a scan of `crates/ph2d-editor-core/src/screens/hero/chrome/*.rs`.
//!
//! ```text
//! cargo run -p ph2d-chrome-sync
//! ```
//!
//! Only the `mod <name>;` declarations are regenerated. The
//! `dispatch_all` chain stays hand-written (z-order is load-bearing).

use ph2d_chrome_sync::{MOD_BEGIN, MOD_END, render_mod_lines, scan_chrome_handlers, splice_lines};
use std::path::Path;

fn main() {
    let chrome_dir = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../crates/ph2d-editor-core/src/screens/hero/chrome")
        .canonicalize()
        .expect("chrome dir resolves");

    let handlers = scan_chrome_handlers(&chrome_dir);
    let mod_file = chrome_dir.join("mod.rs");

    let content = std::fs::read_to_string(&mod_file).expect("chrome/mod.rs readable");
    let updated = splice_lines(&content, MOD_BEGIN, MOD_END, &render_mod_lines(&handlers));
    std::fs::write(&mod_file, updated).expect("chrome/mod.rs writable");

    println!(
        "ph2d-chrome-sync: {} chrome handler(s) synced (mod declarations only).",
        handlers.len(),
    );
}
