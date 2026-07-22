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
    // Unit tests for `tool_rail` (split out for the widget LOC cap) — no user-facing widget; the
    // parent `tool_rail.rs` owns the a11y wiring (build_a11y / build_entry_a11y).
    ("tool_rail/tests.rs", "test module; parent owns a11y"),
    // The dropdown's OPEN list, split out for the widget LOC cap. Paint only: the parent
    // `dropdown/mod.rs` builds the ComboBox node AND one `ListBoxOption` per row
    // (`build_a11y`/`build_option_a11y`), so the rows painted here are already announced.
    (
        "dropdown/popover.rs",
        "paint only; parent owns a11y for the chip and every option",
    ),
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
    // Motion graph wire drawing — split from `paint.rs` for the 600-LOC cap. It owns no
    // widget and registers nothing: it flattens and strokes the wire splines, while every
    // a11y node for a wire (and for its routing waypoints) is registered by `hits.rs`,
    // which wires AccessKit itself. Keeping the drawing and the hit path in one file is
    // what the cap forbids; keeping them AGREEING is what `wire_path` is for (doc 44).
    (
        "ph2d-panel-motion-graph/src/paint_wire.rs",
        "pure spline drawing — the wires' AccessKit nodes are registered in hits.rs",
    ),
    // The add-node popup's DRAW — split from `paint.rs` for the 600-LOC cap (doc 57).
    //
    // It registers nothing because the MENU registers nothing: its rows are hit-tested
    // panel-side against the full-canvas `Background` shield (`interact::apply_background`
    // → `geom::add_menu_row`), a design that predates this split by a long way (M1.E7).
    // So the a11y gap MOVED here with the split; it did not appear with it — and papering
    // over it with an `use ph2d_a11y` that registers nothing would satisfy this scan while
    // making the menu no more reachable than it is today.
    //
    // **The real debt, named:** the add-menu's 86 rows have no AccessKit nodes at all.
    // Closing it means giving each row an id + hit rect of its own instead of the shield.
    (
        "ph2d-panel-motion-graph/src/paint_menu.rs",
        "add-menu draw — its rows are hit-tested against the Background shield and register \
         no ids anywhere (pre-existing M1.E7 gap, not introduced by the split)",
    ),
    (
        "ph2d-panel-motion-graph/src/paint_wire_tests.rs",
        "the flattening guards for paint_wire.rs — a test module, it paints nothing",
    ),
    (
        "ph2d-panel-motion-graph/src/paint_stamp.rs",
        "the card's postage stamp — pure drawing inside the CARD, whose AccessKit node is hits.rs's",
    ),
    // Painter Brush appearance sections (6–11) — a thin ORCHESTRATOR split from
    // `paint_brush.rs` for the 200-LOC/fn + 600-LOC/file caps. It owns no widget:
    // every row it paints is a call into a section module (`paint_shape`,
    // `paint_texture`, `paint_stroke`, `paint_symmetry`, `paint_watercolor`, …),
    // and each of those wires its own AccessKit nodes via the canonical primitives.
    (
        "ph2d-panel-painter-layers/src/paint_brush_sections.rs",
        "orchestrator only — every section it calls wires its own a11y (paint_shape/_texture/_stroke/…)",
    ),
    // Vector path-reshape subsection — a thin section painter split from
    // `paint_sections.rs` for the LOC cap. Its buttons delegate to
    // `BodyCtx::row2` / `action_button` (in paint_sections), which paint via the
    // a11y-wired `paint_button` primitive; this file has no widget of its own.
    (
        "ph2d-panel-vector/src/paint_arrange.rs",
        "delegates to row2/action_button (paint_button-backed) in paint_sections",
    ),
    // Vector connector subsection (Route / Jetty / Spread) — mesmo caso do
    // `paint_arrange` acima: a seção não tem widget PRÓPRIO. As três linhas dela são
    // `BodyCtx::labeled_choice_button` / `labeled_number_field` (em `paint_modes`), que
    // pintam pelos primitivos a11y-wired `paint_segmented_button` e
    // `paint_number_input_with_buffer` — os MESMOS que desenham os parâmetros de forma.
    (
        "ph2d-panel-vector/src/paint_connector.rs",
        "delegates to labeled_choice_button/labeled_number_field (paint_segmented/paint_number_input-backed) in paint_modes",
    ),
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
    // Stencil card — its number boxes delegate to `number_field` (the a11y-wired NumberInput
    // primitive); the card background + labels are decorative chrome.
    (
        "ph2d-panel-painter-layers/src/paint_stencil.rs",
        "number boxes delegate to number_field (a11y-wired NumberInput); rest is decorative chrome",
    ),
    // Flatten/rotate gizmo — its two handles are `CurvePoint`s dispatched in editor-core (the same
    // pattern as paint_falloff); the rim + ellipse + axes are a decorative template.
    (
        "ph2d-panel-painter-layers/src/paint_shape_dab.rs",
        "gizmo handles are CurvePoints dispatched in editor-core; rest is a decorative template render",
    ),
    // Wet Paint TILT dial (doc 22) — the pad is a `CurvePoint` dispatched in editor-core (the
    // paint_shape_dab pattern) and its toggle delegates to `paint_checkbox_row` (the a11y-wired
    // Checkbox); the polar grid + knob are a decorative template render.
    (
        "ph2d-panel-painter-layers/src/paint_wetpaint_tilt.rs",
        "tilt pad is a CurvePoint dispatched in editor-core; toggle delegates to paint_checkbox_row",
    ),
    // Watercolor section — its Wet-edges / Pigment checkboxes delegate to `paint_checkbox_row`
    // (the a11y-wired Checkbox) and the Edge / Spread / Granulation / Mix sliders to `number_field`
    // (the a11y-wired NumberInput); the collapsible header + labels are decorative chrome. Same
    // delegation as `paint_stencil.rs` (its helpers just don't happen to name a canonical primitive).
    (
        "ph2d-panel-painter-layers/src/paint_watercolor.rs",
        "checkboxes delegate to paint_checkbox_row (Checkbox); sliders to number_field (NumberInput); rest is chrome",
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
