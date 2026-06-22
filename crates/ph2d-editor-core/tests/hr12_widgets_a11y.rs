//! HR-12 enforcement: every widget file under `ph2d-editor/src/widget/`
//! must wire AccessKit nodes.
//!
//! Heuristic: file imports something from `ph2d_a11y` (which exposes
//! `Node`/`NodeBuilder`/`Role`/`Action`/`NodeId` — the types every
//! widget builds its a11y node out of). Sufficient because the
//! codebase has no separate `Accessible` trait today; the contract is
//! "you import a11y and emit a `Node` from your `build_a11y` method."
//!
//! A widget that genuinely needs no a11y surface (pure paint helper
//! module, no independent user-facing semantics) must be added to
//! `A11Y_OPT_OUT` below with a 1-line justification.
//!
//! Failure modes this catches:
//! - Adding a new widget without a11y wiring.
//! - Removing a11y wiring from an existing widget.
//!
//! Both are intentional review gates — to fix, either wire a11y or
//! add an explicit opt-out entry.

use std::fs;
use std::path::{Path, PathBuf};

/// Files that legitimately don't need a11y wiring of their own.
/// Each entry: (relative path under `src/widget/`, justification).
const A11Y_OPT_OUT: &[(&str, &str)] = &[
    // BlenderColorPicker sub-components: the parent `mod.rs` owns
    // the a11y tree for the whole picker. These four files are paint
    // helpers and state structs with no standalone user-facing
    // semantics. Re-evaluate if any becomes an independently
    // addressable widget.
    (
        "blender_color_picker/channels.rs",
        "paint helper; parent mod owns a11y",
    ),
    (
        "blender_color_picker/hex_field.rs",
        "paint helper; parent mod owns a11y",
    ),
    (
        "blender_color_picker/value_slider.rs",
        "paint helper; parent mod owns a11y",
    ),
    (
        "blender_color_picker/wheel.rs",
        "paint helper; parent mod owns a11y",
    ),
    // Wave 8 Phase 2.A panel chrome: shared paint helpers + constants
    // (paint_panel_surface, drag/resize hit-zone rects, clamp math,
    // HIGHLIGHTER_RGBA). No standalone user-facing semantics — each
    // panel that uses these owns its own a11y tree via its parent
    // panel manifest.
    (
        "panel_chrome.rs",
        "shared paint helpers; consumer panel owns a11y",
    ),
    // Wave 8 Phase 2.A widget gallery showcase tree: 10 section
    // painters + body orchestrator + state thread-locals. The
    // showcase paints reference widgets which DO emit a11y nodes
    // via the widget primitives they call (paint_button etc.); the
    // sections themselves are paint orchestration with no
    // independent user-facing identity. Owner panel
    // (ph2d-panel-widget-gallery) carries the a11y root.
    (
        "showcase/mod.rs",
        "paint orchestrator; owner panel carries a11y",
    ),
    ("showcase/actions.rs", "section painter; widgets emit a11y"),
    (
        "showcase/body.rs",
        "showcase orchestrator; widgets emit a11y",
    ),
    ("showcase/card.rs", "section painter; widgets emit a11y"),
    ("showcase/color.rs", "section painter; widgets emit a11y"),
    ("showcase/identity.rs", "section painter; widgets emit a11y"),
    ("showcase/inputs.rs", "section painter; widgets emit a11y"),
    (
        "showcase/inspector_w6.rs",
        "section painter; widgets emit a11y",
    ),
    ("showcase/lists.rs", "section painter; widgets emit a11y"),
    ("showcase/notes.rs", "note painter; TextInput emits a11y"),
    ("showcase/slider.rs", "section painter; widgets emit a11y"),
    (
        "showcase/state.rs",
        "thread-locals; no user-facing identity",
    ),
    ("showcase/status.rs", "section painter; widgets emit a11y"),
    ("showcase/switches.rs", "section painter; widgets emit a11y"),
    ("showcase/vector.rs", "section painter; widgets emit a11y"),
    // Canonical icon-button: a pure draw fn. The consumer chrome
    // (TopBar) registers each button's hit-rect AND its AccessKit node,
    // same split as panel_chrome / the showcase section painters.
    (
        "icon_button.rs",
        "paint helper; consumer chrome (TopBar) owns hit + a11y",
    ),
];

#[test]
fn every_widget_file_wires_a11y() {
    let crate_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let widget_root = crate_root.join("src/widget");
    let opt_out: Vec<&str> = A11Y_OPT_OUT.iter().map(|(p, _)| *p).collect();
    let mut violations = Vec::new();
    walk(&widget_root, &widget_root, &mut |relpath, abspath| {
        if abspath.extension().and_then(|s| s.to_str()) != Some("rs") {
            return;
        }
        let rel = relpath.to_string_lossy().replace('\\', "/");
        // Top-level mod.rs is re-exports only; sub-mod.rs files in
        // composite widgets (e.g. blender_color_picker/mod.rs) ARE
        // checked because that's where the widget's a11y root lives.
        if rel == "mod.rs" {
            return;
        }
        if opt_out.contains(&rel.as_str()) {
            return;
        }
        let content = fs::read_to_string(abspath).expect("read widget file");
        let has_a11y = content.contains("use ph2d_a11y") || content.contains("ph2d_a11y::");
        if !has_a11y {
            violations.push(rel);
        }
    });

    // Wave 10 / Etapa 5.1: extend scope to panel-* crates. Panel files
    // either (a) wire a11y directly (panels with a custom a11y root —
    // e.g. composite chrome), OR (b) DELEGATE all a11y to widget
    // primitives they call (paint_button, paint_toggle, paint_slider…
    // — every widget primitive owns its own AccessKit emission). To
    // make this delegation explicit AND testable, the gate accepts
    // either: `use ph2d_a11y` import, OR a call to a canonical widget
    // primitive (`paint_button`, `paint_toggle`, etc.). Files that
    // satisfy neither (pure-paint helpers with no widget interaction)
    // go on PANEL_DELEGATE_OK below with a one-line justification.
    let crates_root = crate_root.join("..");
    let panel_delegate_ok: &[(&str, &str)] = PANEL_A11Y_DELEGATE_OK;
    let delegate_ok_paths: Vec<&str> = panel_delegate_ok.iter().map(|(p, _)| *p).collect();
    if let Ok(entries) = fs::read_dir(&crates_root) {
        let mut panel_dirs: Vec<PathBuf> = entries
            .flatten()
            .filter_map(|e| {
                let path = e.path();
                let name = path.file_name()?.to_str()?.to_string();
                if path.is_dir()
                    && name.starts_with("ph2d-panel-")
                    && name != "ph2d-panel-registry-init"
                {
                    Some(path)
                } else {
                    None
                }
            })
            .collect();
        panel_dirs.sort();
        for panel_dir in &panel_dirs {
            let src = panel_dir.join("src");
            if !src.is_dir() {
                continue;
            }
            let crate_name = panel_dir
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("")
                .to_string();
            walk(&src, &src, &mut |relpath, abspath| {
                if abspath.extension().and_then(|s| s.to_str()) != Some("rs") {
                    return;
                }
                let rel = relpath.to_string_lossy().replace('\\', "/");
                // Skip non-paint panel files. State/event/id/populate/sync
                // are panel internals that DON'T paint UI (state machine,
                // input dispatch, NodeId tables, store init, value-sync).
                // The paint-orchestrator files (paint*.rs) are where a11y
                // delegation must surface.
                let base = relpath.file_name().and_then(|n| n.to_str()).unwrap_or("");
                let is_paint_file = base.starts_with("paint")
                    || base == "sections.rs"  // inspector sections paint
                    || base == "paint_kinds.rs"
                    || base == "paint_rows.rs"
                    || base == "paint_helpers.rs";
                if !is_paint_file {
                    return;
                }
                if rel == "mod.rs" || rel == "lib.rs" {
                    return; // re-export / glue
                }
                let key = format!("{crate_name}/src/{rel}");
                if delegate_ok_paths.contains(&key.as_str()) {
                    return;
                }
                let content = fs::read_to_string(abspath).expect("read panel file");
                let has_direct_a11y =
                    content.contains("use ph2d_a11y") || content.contains("ph2d_a11y::");
                // Canonical widget primitives — calling these wires a11y
                // transitively (each owns its own AccessKit emission).
                let delegates_to_widgets =
                    WIDGET_DELEGATE_MARKERS.iter().any(|m| content.contains(m));
                if !has_direct_a11y && !delegates_to_widgets {
                    violations.push(key);
                }
            });
        }
    }

    assert!(
        violations.is_empty(),
        "HR-12 violation — widget/panel files without AccessKit wiring \
         (must `use ph2d_a11y::...` OR delegate to a canonical widget \
         primitive: {WIDGET_DELEGATE_MARKERS:?}):\n  {}\n\n\
         If a file genuinely has no user-facing semantics, add it to \
         A11Y_OPT_OUT (widget files) or PANEL_A11Y_DELEGATE_OK \
         (panel files) in this test with a 1-line justification.",
        violations.join("\n  "),
    );
}

/// Canonical widget primitives. Calling any of these inside a panel
/// file means a11y is wired transitively (the primitive owns its own
/// AccessKit emission). Keep in sync with `src/widget/` paint helpers.
const WIDGET_DELEGATE_MARKERS: &[&str] = &[
    "paint_button",
    "paint_toggle",
    "paint_slider",
    "paint_number_input",
    "paint_text_input",
    "paint_color_swatch",
    "paint_list_item",
    "paint_chip",
    "paint_icon_button",
    "paint_segmented",
    "paint_dropdown",
    "paint_popover",
    "paint_card",
    "panel_chrome::",
    "paint_panel_title",
    "paint_panel_surface",
];

/// Panel files that paint via vector/text primitives only (no widget
/// interaction → no a11y to wire). Each entry: (path key, why).
const PANEL_A11Y_DELEGATE_OK: &[(&str, &str)] = &[
    // CEQ histogram strip — pure RGB-bar visualization (read-only chart,
    // no widget interaction, no AccessKit semantics). Split out from
    // `paint.rs` to satisfy Wave 10 LOC cap.
    (
        "ph2d-panel-color-equalization/src/paint_histogram.rs",
        "read-only histogram visualization, no a11y semantics",
    ),
    // Falloff curve editor — render half only. Its interactive elements get their
    // a11y from the registered-widget system, not this file: the +/− buttons are
    // registered widgets (populate + event drain), and the draggable curve handles
    // are dispatched in editor-core (the 2D-drag BlenderHit pattern).
    // TODO(a11y follow-up): wire AccessKit nodes for the curve handles themselves.
    (
        "ph2d-panel-painter-layers/src/paint_falloff.rs",
        "falloff-curve render half; handles dispatched in editor-core, buttons are registered widgets",
    ),
];

fn walk(root: &Path, dir: &Path, cb: &mut dyn FnMut(&Path, &Path)) {
    for entry in fs::read_dir(dir).expect("read widget dir") {
        let entry = entry.expect("dir entry");
        let path = entry.path();
        if path.is_dir() {
            walk(root, &path, cb);
        } else if let Ok(rel) = path.strip_prefix(root) {
            cb(rel, &path);
        }
    }
}
