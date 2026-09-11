//! Arch-gate: every panel that publishes a **scrollbar thumb** must also
//! intercept the **mouse wheel**.
//!
//! ## The bug this exists to prevent
//!
//! Making a panel scrollable takes four separate edits, and only three of them
//! fail loudly:
//!
//! 1. a thumb `NodeId` in `editor-core/src/widget/scrollbar.rs`,
//! 2. an arm in `scrollbar_panel_for_id` (`interaction/dispatch/scroll.rs`) so the
//!    thumb DRAG routes to the panel,
//! 3. the painter reading `panel_scroll` + publishing `content_h`/`visible_h`,
//! 4. the panel id in `cursor_over_hero_panel` (`shells/desktop/src/forwarding.rs`)
//!    so the WHEEL reaches `dispatch_wheel` instead of zooming the camera.
//!
//! Forget (4) and the panel still compiles, still paints its scrollbar, and the
//! thumb still drags — but the wheel silently zooms the camera underneath. The
//! Audio Mixer shipped like that and nobody noticed (found 2026-07-09, while
//! wiring the Audio Editor's scroll).
//!
//! ## How it works
//!
//! `scrollbar_panel_for_id` is the single source of truth for "this panel scrolls".
//! Parse the panel ids out of it, then assert each one is named inside the body of
//! `cursor_over_hero_panel`. Both are plain source scans — no runtime needed.

use std::fs;
use std::path::PathBuf;

/// Panel ids in the dispatch map that have **no live panel** to intercept a wheel
/// for. Keep each entry justified; an unjustified entry is a hidden regression.
const ALLOWLIST: &[(&str, &str)] = &[(
    "PAINTER_BRUSH_STUDIO_PANEL",
    "crate `ph2d-panel-brush-studio` was deleted by ADR-0099; the id + dispatch arm \
     are vestigial and nothing publishes a rect for it",
)];

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
}

fn read(rel: &str) -> String {
    let path = repo_root().join(rel);
    fs::read_to_string(&path).unwrap_or_else(|e| panic!("cannot read {}: {e}", path.display()))
}

/// The `*_PANEL` identifiers `scrollbar_panel_for_id` maps its thumb ids onto.
fn scrollable_panel_ids(src: &str) -> Vec<String> {
    let start = src
        .find("fn scrollbar_panel_for_id")
        .expect("scrollbar_panel_for_id vanished — this gate is stale");
    let body = &src[start..];
    let end = body.find("\n}").expect("unterminated fn");
    let mut out = Vec::new();
    for line in body[..end].lines() {
        // Arms look like `Some(ids::INSP_PANEL)` or `Some(crate::ids::GS_PANEL)`.
        let Some(open) = line.find("Some(") else {
            continue;
        };
        let Some(close) = line[open..].find(')') else {
            continue;
        };
        let ident = line[open + "Some(".len()..open + close]
            .rsplit("::")
            .next()
            .unwrap_or("")
            .trim();
        if ident.ends_with("_PANEL") && !out.iter().any(|p| p == ident) {
            out.push(ident.to_string());
        }
    }
    assert!(
        out.len() > 5,
        "parsed only {} panel ids — the parser broke, not the wiring",
        out.len()
    );
    out
}

/// The panel ids `cursor_over_hero_panel` actually tests, i.e. the arguments of its
/// `inside(...)` calls.
///
/// Deliberately NOT a substring scan of the whole function: its body opens with a
/// `use ...::ids::{INSP_PANEL, ...}` import, so "the name appears somewhere in the
/// function" is satisfied by the import alone. A gate that green-lights a panel
/// whose `inside(...)` call was deleted is worse than no gate — this one is
/// mutation-tested by deleting an `|| inside(X)` line and watching it fail.
fn wheel_intercepted_panel_ids(src: &str) -> Vec<String> {
    let start = src
        .find("pub fn cursor_over_hero_panel")
        .expect("cursor_over_hero_panel vanished — this gate is stale");
    let body = &src[start..];
    let end = body.find("\n}").expect("unterminated fn");
    let body = &body[..end];

    let mut out = Vec::new();
    let mut rest = body;
    // Skip the closure's own definition (`let inside = |panel_id| {...}`).
    while let Some(at) = rest.find("inside(") {
        let arg_start = at + "inside(".len();
        let Some(close) = rest[arg_start..].find(')') else {
            break;
        };
        let ident = rest[arg_start..arg_start + close]
            .rsplit("::")
            .next()
            .unwrap_or("")
            .trim();
        if ident.ends_with("_PANEL") && !out.iter().any(|p| p == ident) {
            out.push(ident.to_string());
        }
        rest = &rest[arg_start + close..];
    }
    out
}

#[test]
fn every_scrollable_panel_intercepts_the_wheel() {
    let ids = scrollable_panel_ids(&read(
        "crates/ph2d-editor-core/src/interaction/dispatch/scroll.rs",
    ));
    let intercepted = wheel_intercepted_panel_ids(&read("shells/desktop/src/forwarding.rs"));
    assert!(
        intercepted.len() > 5,
        "parsed only {} `inside(...)` calls — the parser broke, not the wiring",
        intercepted.len()
    );

    let missing: Vec<&String> = ids
        .iter()
        .filter(|id| !ALLOWLIST.iter().any(|(a, _)| *a == id.as_str()))
        .filter(|id| !intercepted.contains(id))
        .collect();

    assert!(
        missing.is_empty(),
        "these panels publish a scrollbar thumb but do NOT intercept the wheel, so \
         wheeling over them zooms the camera instead of scrolling: {missing:?}\n\
         Fix: add `|| inside({})` to `cursor_over_hero_panel` in \
         shells/desktop/src/forwarding.rs (and import the id).",
        missing.first().map(|s| s.as_str()).unwrap_or("X_PANEL"),
    );
}

/// An allowlist entry that no longer appears in the dispatch map is dead weight —
/// and worse, it would silently excuse a *new* panel that happened to reuse the name.
#[test]
fn the_allowlist_has_no_stale_entries() {
    let ids = scrollable_panel_ids(&read(
        "crates/ph2d-editor-core/src/interaction/dispatch/scroll.rs",
    ));
    for (allowed, why) in ALLOWLIST {
        assert!(
            ids.iter().any(|id| id == allowed),
            "allowlist entry `{allowed}` is no longer in `scrollbar_panel_for_id` — \
             drop it. (It was excused because: {why})"
        );
    }
}
