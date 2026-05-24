//! Wave 10 / Etapa 4.2 — staleness gate for `ph2d-chrome-sync`.
//!
//! Two sub-checks:
//!
//! 1. `chrome/mod.rs`'s `mod foo;` block matches a fresh scan of the
//!    chrome dir (i.e. `cargo run -p ph2d-chrome-sync` was rerun after
//!    a handler was added/removed).
//!
//! 2. Every `mod`'d handler is ALSO referenced in the `dispatch_all`
//!    chain. The chain is hand-written (z-order load-bearing) — this
//!    catches the bug "added handler file + ran sync, but forgot to
//!    wire it into dispatch_all".

use ph2d_chrome_sync::{
    MOD_BEGIN, MOD_END, dispatch_all_referenced_handlers, render_mod_lines, scan_chrome_handlers,
    splice_lines,
};
use std::path::PathBuf;

fn chrome_mod_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src/screens/hero/chrome/mod.rs")
}

fn chrome_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src/screens/hero/chrome")
}

#[test]
fn chrome_mod_block_in_sync_with_folder() {
    let path = chrome_mod_path();
    let handlers = scan_chrome_handlers(&chrome_dir());
    let on_disk = std::fs::read_to_string(&path).expect("chrome/mod.rs readable");
    let rendered = splice_lines(&on_disk, MOD_BEGIN, MOD_END, &render_mod_lines(&handlers));
    assert_eq!(
        on_disk, rendered,
        "chrome/mod.rs `mod` declaration block is stale.\n\
         Run `cargo run -p ph2d-chrome-sync` and commit the regenerated file."
    );
}

#[test]
fn every_mod_handler_is_in_dispatch_all() {
    let path = chrome_mod_path();
    let handlers = scan_chrome_handlers(&chrome_dir());
    let content = std::fs::read_to_string(&path).expect("chrome/mod.rs readable");
    let dispatched = dispatch_all_referenced_handlers(&content);
    let missing: Vec<String> = handlers
        .iter()
        .filter(|h| !dispatched.contains(h))
        .cloned()
        .collect();
    assert!(
        missing.is_empty(),
        "Handler module(s) exist in chrome/ but are NOT referenced in dispatch_all:\n  {}\n\
         \n\
         Add `<name>::apply(hero, event)` at the appropriate z-order\n\
         position in dispatch_all (the chain is hand-written because\n\
         z-order is load-bearing per chrome/mod.rs doc-comment).",
        missing.join(", ")
    );
}
