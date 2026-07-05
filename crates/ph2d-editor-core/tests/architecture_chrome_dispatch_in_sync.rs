//! Wave 10 / Etapa 4.2 — staleness gate for `ph2d-chrome-sync`.
//! ADR-0107 (Camada 0): `dispatch_all` is now GENERATED too.
//!
//! Two sub-checks, one per generated block:
//!
//! 1. `chrome/mod.rs`'s `mod foo;` block matches a fresh scan of the
//!    chrome dir.
//!
//! 2. `chrome/mod.rs`'s `dispatch_all` `||` chain matches a fresh render
//!    ordered by each handler's `// ph2d-chrome-sync:z=NN` marker. This
//!    subsumes the old "every handler is wired" check (a fresh render
//!    always includes every scanned handler) AND catches order drift.
//!    Both blocks are pure functions of the handler set → concurrent
//!    handler additions never collide (the point of Camada 0).

use ph2d_chrome_sync::{
    DISPATCH_BEGIN, DISPATCH_END, MOD_BEGIN, MOD_END, dispatch_all_referenced_handlers,
    render_dispatch_lines, render_mod_lines, scan_chrome_handlers, sorted_dispatch_handlers,
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
fn dispatch_all_chain_in_sync_with_folder() {
    let path = chrome_mod_path();
    let sorted = sorted_dispatch_handlers(&chrome_dir());
    let on_disk = std::fs::read_to_string(&path).expect("chrome/mod.rs readable");
    let rendered = splice_lines(
        &on_disk,
        DISPATCH_BEGIN,
        DISPATCH_END,
        &render_dispatch_lines(&sorted),
    );
    assert_eq!(
        on_disk, rendered,
        "chrome/mod.rs `dispatch_all` chain is stale.\n\
         Run `cargo run -p ph2d-chrome-sync` and commit the regenerated file.\n\
         Chain order = each handler's `// ph2d-chrome-sync:z=NN` marker, then name."
    );
}

#[test]
fn every_handler_appears_in_dispatch_all() {
    // Belt-and-suspenders on top of the staleness check: a fresh render
    // must reference every scanned handler. Guards against a future
    // render bug silently dropping a handler from the chain.
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
        "Handler(s) in chrome/ but NOT in dispatch_all: {}\n\
         Run `cargo run -p ph2d-chrome-sync`.",
        missing.join(", ")
    );
}
