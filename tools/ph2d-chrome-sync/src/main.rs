#![forbid(unsafe_code)]
//! `ph2d-chrome-sync` — regenerate BOTH generated blocks in
//! `crates/ph2d-editor-core/src/screens/hero/chrome/mod.rs` from a scan of
//! the sibling `chrome/*.rs` handler files:
//!   1. the `mod <name>;` declaration block, and
//!   2. the `dispatch_all` `||` chain body — ordered by each handler's
//!      `// ph2d-chrome-sync:z=NN` marker, then name (ADR-0107, Camada 0).
//!
//! ```text
//! cargo run -p ph2d-chrome-sync
//! ```
//!
//! Both blocks are pure functions of the handler set → concurrent handler
//! additions from parallel lines never collide on the shared file.

use ph2d_chrome_sync::{
    DISPATCH_BEGIN, DISPATCH_END, MOD_BEGIN, MOD_END, render_dispatch_lines, render_mod_lines,
    scan_chrome_handlers, sorted_dispatch_handlers, splice_lines,
};
use std::path::Path;

fn main() {
    let chrome_dir = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../crates/ph2d-editor-core/src/screens/hero/chrome")
        .canonicalize()
        .expect("chrome dir resolves");

    let handlers = scan_chrome_handlers(&chrome_dir);
    let sorted = sorted_dispatch_handlers(&chrome_dir);
    let mod_file = chrome_dir.join("mod.rs");

    let content = std::fs::read_to_string(&mod_file).expect("chrome/mod.rs readable");
    // Splice the mod block, then the dispatch block, on the same content.
    let content = splice_lines(&content, MOD_BEGIN, MOD_END, &render_mod_lines(&handlers));
    let content = splice_lines(
        &content,
        DISPATCH_BEGIN,
        DISPATCH_END,
        &render_dispatch_lines(&sorted),
    );
    std::fs::write(&mod_file, content).expect("chrome/mod.rs writable");

    println!(
        "ph2d-chrome-sync: {} handler(s) synced (mod block + dispatch_all chain).",
        handlers.len(),
    );
}
