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
];

#[test]
fn every_widget_file_wires_a11y() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src/widget");
    let opt_out: Vec<&str> = A11Y_OPT_OUT.iter().map(|(p, _)| *p).collect();
    let mut violations = Vec::new();
    walk(&root, &root, &mut |relpath, abspath| {
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
    assert!(
        violations.is_empty(),
        "HR-12 violation — widget files without AccessKit wiring \
         (must `use ph2d_a11y::...`):\n  {}\n\n\
         If a file genuinely has no user-facing semantics, add it to \
         A11Y_OPT_OUT in this test with a 1-line justification.",
        violations.join("\n  "),
    );
}

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
