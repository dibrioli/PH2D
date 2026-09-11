//! ADR-0029 §6.2 — surface-area gate for the `PanelHost` /
//! `PanelHostInternal` traits.
//!
//! These traits define the entire interface panel crates consume from
//! the host orchestrator. Letting them grow unchecked re-creates the
//! god-trait problem the ADR set out to solve — every new accessor is
//! one more thing 3rd-party panels can depend on, one more entry to
//! stabilize, one more reason to fork. The gate forces explicit review:
//! when a panel migration legitimately needs a new accessor, bump the
//! threshold in this test in the same commit and justify it.
//!
//! Implementation reads `src/panel/host.rs` as text and counts `fn `
//! tokens inside each trait body. Dep-free — `syn`/`proc-macro2`
//! aren't workspace dev-deps. Methods with non-trivial signatures
//! (joint accessors, default-bodied helpers) all count equally — one
//! per public-API surface entry.

use std::fs;
use std::path::{Path, PathBuf};

/// Inclusive cap on `PanelHost` (public-stable tier). ADR-0029 §4.1
/// targets ~8-12 once carved out from `PanelHostInternal` post-6-
/// months. Pre-stabilization the trait is intentionally minimal.
const PANEL_HOST_LIMIT: usize = 12;

/// Inclusive cap on `PanelHostInternal` (`#[doc(hidden)]` unstable
/// tier). ADR-0029 §4.2 expects ~25-30 methods at full panel coverage.
/// 35 gives slack for cross-panel accessors that emerge during
/// post-migration polish without forcing a same-commit threshold bump.
const PANEL_HOST_INTERNAL_LIMIT: usize = 35;

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("crates/")
        .parent()
        .expect("workspace root")
        .to_path_buf()
}

fn host_source() -> String {
    let path = workspace_root()
        .join("crates")
        .join("ph2d-editor-core")
        .join("src")
        .join("panel")
        .join("host.rs");
    fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()))
}

/// Slice the body of `trait <name>` (between its opening `{` and the
/// matching closing `}`). Returns `None` if the trait isn't found.
/// Naive brace counting — fine for the host trait file which has no
/// macros / nested traits.
fn trait_body<'a>(source: &'a str, name: &str) -> Option<&'a str> {
    let needle = format!("trait {name}");
    let start = source.find(&needle)?;
    let after = &source[start + needle.len()..];
    let brace_rel = after.find('{')?;
    let body_start_abs = start + needle.len() + brace_rel + 1;
    let bytes = source.as_bytes();
    let mut depth = 1usize;
    let mut i = body_start_abs;
    while i < bytes.len() {
        match bytes[i] {
            b'{' => depth += 1,
            b'}' => {
                depth -= 1;
                if depth == 0 {
                    return Some(&source[body_start_abs..i]);
                }
            }
            _ => {}
        }
        i += 1;
    }
    None
}

/// Count `fn ` tokens at column-start-of-statement positions inside a
/// trait body. Skips comment lines (`//` after trim) and counts each
/// method declaration exactly once.
fn count_methods(body: &str) -> usize {
    let mut count = 0usize;
    for raw in body.lines() {
        let line = raw.trim_start();
        if line.starts_with("//") {
            continue;
        }
        // `fn ` at the start of a statement covers `fn foo(...)`,
        // `fn foo(...) -> ...`, and trait methods with default
        // bodies (`fn foo(...) -> ... { ... }` on one line). It does
        // NOT match `&fn` / `&dyn Fn` / parameter names — those don't
        // start with `fn ` after the line's leading whitespace.
        if line.starts_with("fn ") {
            count += 1;
        }
    }
    count
}

#[test]
fn panel_host_public_surface_within_limit() {
    let source = host_source();
    let body = trait_body(&source, "PanelHost")
        .expect("`pub trait PanelHost` block not found in panel/host.rs");
    let count = count_methods(body);
    assert!(
        count <= PANEL_HOST_LIMIT,
        "PanelHost surface grew to {count} methods (cap {PANEL_HOST_LIMIT}). \
         ADR-0029 §4.1 keeps this tier minimal — either justify a cap bump in \
         this test (same commit), or move the new method to PanelHostInternal."
    );
}

#[test]
fn panel_host_internal_surface_within_limit() {
    let source = host_source();
    let body = trait_body(&source, "PanelHostInternal")
        .expect("`pub trait PanelHostInternal` block not found in panel/host.rs");
    let count = count_methods(body);
    assert!(
        count <= PANEL_HOST_INTERNAL_LIMIT,
        "PanelHostInternal surface grew to {count} methods (cap \
         {PANEL_HOST_INTERNAL_LIMIT}). ADR-0029 §4.2 expects ~25-30; if a new \
         panel legitimately needs another accessor, bump the cap in this test \
         in the same commit and justify it."
    );
}

/// Self-check the parser — counts shouldn't be zero (we'd silently
/// pass the cap test if `count_methods` started returning 0).
#[test]
fn surface_counter_self_check() {
    let source = host_source();
    let public = trait_body(&source, "PanelHost").expect("PanelHost block");
    let internal = trait_body(&source, "PanelHostInternal").expect("PanelHostInternal block");
    assert!(
        count_methods(public) >= 1,
        "PanelHost should expose at least one method"
    );
    assert!(
        count_methods(internal) >= 5,
        "PanelHostInternal should expose at least five methods"
    );
}
