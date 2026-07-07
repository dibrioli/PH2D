//! Vector tool ⟷ shell bridge (ADR-0108 cutover).
//!
//! Per-frame jobs (mirror of the Padding / Painter panel bridges):
//!
//! 1. **Panel visibility** — show the docked `ph2d-panel-vector` Style panel
//!    (right dock) iff the `vector` tool is active; hide the real Inspector
//!    (edge-triggered) so they don't both claim the slot.
//! 2. **Picker read-back** — a Down on the Stroke / Fill swatch opened the
//!    shared OKLCH picker (generic `is_picker_swatch` dispatch). While the
//!    picker targets one of our swatches, feed the live picked colour into the
//!    tool via [`VectorTool::set_stroke_rgba`] / [`VectorTool::set_fill_rgba`].
//! 3. **Style sync** — copy the tool's stroke / fill / width into the Pen so
//!    newly drawn paths honour the Style.
//! 4. **Recolour selected** — when a colour changed (`take_apply_to_selected`),
//!    recolour the selected path. ONE undo step per gesture: a picker drag
//!    commits on close; a discrete pick (Fill "None") commits the same frame.
//! 5. **Publish** — sync the swatches' `widget_color` to the live colour (seeds
//!    the picker on open) + publish the Style snapshot the panel paints.
//!
//! The concrete-tool downcast lives HERE (allowlisted:
//! `architecture_no_downcast_to_concrete_tool_in_shell`), so the central render
//! loop stays downcast-free — mirror of `painter_bridge`.

use ph2d_editor::{HeroScreen, ToolId, ToolRegistry};
use ph2d_tool_vector::{DEFAULT_POLYGON_SIDES, DrawMode};
use ph2d_vec_edit::{History, PenStyle, PenTool, ShapeTool};
use ph2d_vec_scene::{Rgba8, VecScene};
use std::cell::RefCell;

fn rgba(c: [u8; 4]) -> Rgba8 {
    Rgba8::new(c[0], c[1], c[2], c[3])
}

thread_local! {
    /// Pre-image of the scene captured at the START of a recolour gesture (the
    /// first frame the colour actually changes the selected path). Committed to
    /// `History` as ONE undo step when the gesture ends (the picker closes /
    /// the discrete pick's frame finishes). `None` between gestures.
    static RECOLOR_PRE: RefCell<Option<VecScene>> = const { RefCell::new(None) };
}

/// Per-frame Vector-tool plumbing. Safe to call every frame; a no-op when the
/// Vector tool is absent from the registry.
/// Returns the tool's current `(draw_mode, polygon_sides)` so the shell can
/// mirror them into `App` (the input dispatch reads those to route canvas
/// gestures without a downcast). Defaults to `(Pen, DEFAULT_POLYGON_SIDES)` when
/// the Vector tool is absent.
pub(super) fn dispatch(
    hero: &mut HeroScreen,
    tools: &mut ToolRegistry,
    scene: &mut VecScene,
    pen: &mut PenTool,
    shape: &mut ShapeTool,
    history: &mut History,
    // World units per screen pixel (from the camera) — converts the tool's px
    // stroke width into the path's world-space width when restyling.
    px_to_world: f64,
) -> (DrawMode, u32) {
    let vector_active = tools
        .active()
        .is_some_and(|t| t.id() == ToolId::new("vector"));

    // ── 1. Panel visibility (mirror of the Padding dock takeover) ─────────
    hero.panel_visibility.insert("vector", vector_active);
    {
        use std::sync::atomic::{AtomicBool, Ordering};
        static LAST_ACTIVE: AtomicBool = AtomicBool::new(false);
        let was = LAST_ACTIVE.swap(vector_active, Ordering::Relaxed);
        if was != vector_active {
            hero.panel_visibility.insert("inspector", !vector_active);
        }
    }

    // The tool persists in the registry whether or not it is active, so its
    // Style survives tool switches (mirror of the painter bridge).
    let Some(tool) = tools.tool_by_id_mut(&ToolId::new("vector")).and_then(|t| {
        t.as_any_mut()
            .downcast_mut::<ph2d_tool_vector::VectorTool>()
    }) else {
        #[cfg(feature = "panel-vector")]
        ph2d_panel_vector::set_current_vector_style(None);
        return (DrawMode::Pen, DEFAULT_POLYGON_SIDES);
    };

    // ── 2. Picker read-back: which swatch is the picker targeting? ────────
    let target = hero.store.picker_target();
    let stroke_open = target == Some(ph2d_editor::ids::VECTOR_STROKE_SWATCH);
    let fill_open = target == Some(ph2d_editor::ids::VECTOR_FILL_SWATCH);
    if (stroke_open || fill_open)
        && let Some((value, _, _, _)) = hero
            .store
            .blender_picker(ph2d_editor::ids::INSP_BLENDER_PICKER)
    {
        // A picked colour is always OPAQUE — the picker is an OKLCH colour, and
        // the swatch seed's alpha (0 after "None") must NOT stick, else picking
        // a fill colour after "None" stays invisible (Enio: "none é fixo").
        let c = value.rgba;
        let opaque = [c[0], c[1], c[2], 255];
        if stroke_open {
            tool.set_stroke_rgba(opaque);
        } else {
            tool.set_fill_rgba(opaque);
        }
    }

    let stroke = tool.stroke_rgba();
    let fill = tool.fill_rgba();

    // ── 3. New paths honour the tool's Style (pen + shape share it). ──────
    let style = PenStyle {
        stroke: rgba(stroke),
        stroke_w_px: tool.stroke_width_px(),
        fill: rgba(fill),
    };
    pen.set_style(style);
    shape.set_style(style);

    // ── 4. Restyle the selected path — colour + width (undoable, one step per
    //    gesture). A width-slider DRAG is a gesture like a picker drag, so scope
    //    its undo the same way (one step per drag). Width follows the tool ONLY
    //    while the slider is dragged, so a plain colour pick never resizes it.
    let width_dragging = matches!(
        hero.store.slider(ph2d_editor::ids::VECTOR_WIDTH),
        Some((ph2d_editor::widget::SliderState::Dragging, _))
    );
    let session = stroke_open || fill_open || width_dragging;
    if tool.take_apply_to_selected()
        && let Some(sel) = pen.selected()
    {
        let new_stroke = rgba(stroke);
        let new_fill = if fill[3] == 0 { None } else { Some(rgba(fill)) };
        let new_w = tool.stroke_width_px() * px_to_world;
        let will_change = scene.paths().iter().find(|p| p.id == sel).is_some_and(|p| {
            let colour_differs = matches!(p.stroke, Some((c, _)) if c != new_stroke);
            let width_differs = width_dragging
                && matches!(p.stroke, Some((_, w)) if (w - new_w).abs() > f64::EPSILON);
            let fill_differs = p.closed && p.fill != new_fill;
            colour_differs || width_differs || fill_differs
        });
        if will_change {
            RECOLOR_PRE.with(|c| {
                if c.borrow().is_none() {
                    *c.borrow_mut() = Some(scene.clone());
                }
            });
            if let Some(path) = scene.path_mut(sel) {
                if let Some((_, old_w)) = path.stroke {
                    let w = if width_dragging { new_w } else { old_w };
                    path.stroke = Some((new_stroke, w));
                }
                if path.closed {
                    path.fill = new_fill;
                }
            }
        }
    }
    // Commit the gesture's undo when it ends (no picker / width-drag session):
    // a discrete pick (None) commits immediately; a drag commits on release.
    if !session {
        RECOLOR_PRE.with(|c| {
            if let Some(pre) = c.borrow_mut().take() {
                history.push_undo(pre);
            }
        });
    }

    // ── 5. Sync swatch colours (seeds the picker on open) + publish. ──────
    hero.store
        .set_widget_color(ph2d_editor::ids::VECTOR_STROKE_SWATCH, stroke);
    hero.store
        .set_widget_color(ph2d_editor::ids::VECTOR_FILL_SWATCH, fill);
    #[cfg(feature = "panel-vector")]
    ph2d_panel_vector::set_current_vector_style(if vector_active {
        Some(tool.ui_snapshot())
    } else {
        None
    });

    // Mirror the tool's draw-mode + polygon sides so the input dispatch can
    // route canvas gestures (pen vs shape) without downcasting the tool.
    (tool.mode(), tool.polygon_sides())
}
