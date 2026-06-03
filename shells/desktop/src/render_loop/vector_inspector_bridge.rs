//! Vector Inspector panel ⟷ shell bridge (W2.T2.4).
//!
//! Three per-frame jobs (mirror of the painter bridge's panel plumbing):
//! 1. **Visibility** — show the vector inspector (right-dock) while a vector
//!    selection tool (`vector_select` / `vector_direct`) is active; hide the
//!    real Inspector (edge-triggered) so they don't both claim the slot.
//! 2. **Picker read-back** — a Down on the fill swatch opened the shared Blender
//!    picker (generic `is_picker_swatch` dispatch). While the picker targets the
//!    swatch, mirror the live picked sRGB8 into the swatch (`set_widget_color`)
//!    and `App.vector_fill_color`.
//! 3. **Publish** — push the current fill color to the panel so it paints the
//!    swatch fill.
//!
//! The **apply** (fill color → the selected regions, logged for undo) is the
//! Vector implementer's `apply-fill` part (T2.4 division). It joins HERE: pass
//! `&mut committed` + `&selection` and call the impl's helper inside the read-
//! back block once delivered.

use ph2d_editor::HeroScreen;
use ph2d_editor::ToolRegistry;
use ph2d_vector_doc::{
    Ph2dVectorAsset, VectorSelection, apply_fill_to_selection, preview_fill_on_selection,
};

/// Per-frame vector inspector plumbing. Safe to call every frame.
pub(super) fn dispatch(
    hero: &mut HeroScreen,
    tools: &mut ToolRegistry,
    vector_fill_color: &mut [u8; 4],
    committed: &mut [Ph2dVectorAsset],
    selection: &VectorSelection,
    undo: &mut Vec<Vec<Ph2dVectorAsset>>,
    redo: &mut Vec<Vec<Ph2dVectorAsset>>,
) {
    let vector_active = tools
        .active()
        .map(|t| {
            let id = t.id();
            id == ph2d_editor::ToolId::new("vector_select")
                || id == ph2d_editor::ToolId::new("vector_direct")
                // Shape too (§4.2): it hosts the Shape-kind picker + shares the
                // fill swatch (new shapes take the current fill color).
                || id == ph2d_editor::ToolId::new("vector_shape")
        })
        .unwrap_or(false);

    // ── Visibility (mirror of the painter sidebar takeover) ───────────────
    hero.panel_visibility
        .insert("vector_inspector", vector_active);
    {
        use std::sync::atomic::{AtomicBool, Ordering};
        static LAST_ACTIVE: AtomicBool = AtomicBool::new(false);
        let was = LAST_ACTIVE.swap(vector_active, Ordering::Relaxed);
        if was != vector_active {
            // Edge-triggered: free/restore the shared Inspector slot.
            hero.panel_visibility.insert("inspector", !vector_active);
        }
    }

    // ── Picker read-back: live picked color → swatch + App fill color ─────
    let picker_open =
        hero.store.picker_target() == Some(ph2d_editor::ids::VECTOR_INSPECTOR_FILL_SWATCH);
    if picker_open
        && let Some((value, _, _, _)) = hero
            .store
            .blender_picker(ph2d_editor::ids::INSP_BLENDER_PICKER)
    {
        *vector_fill_color = value.rgba;
        hero.store
            .set_widget_color(ph2d_editor::ids::VECTOR_INSPECTOR_FILL_SWATCH, value.rgba);
    }

    // ── APPLY JOIN (T2.4): REAL-TIME recolor of the selected regions ──────
    // While the fill picker is open, the canvas tracks the picker live. The
    // FIRST frame does one logged `apply_fill_to_selection` (a single undoable
    // `SetRegionFill` that establishes the region→fill assignment); every
    // subsequent frame recolors that fill slot IN PLACE via
    // `preview_fill_on_selection` (no new fill, no new op) — so the preview is
    // real-time without spamming the style table / edit_log per drag-frame.
    {
        use std::sync::atomic::{AtomicBool, Ordering};
        static PICKER_WAS_OPEN: AtomicBool = AtomicBool::new(false);
        static ESTABLISHED: AtomicBool = AtomicBool::new(false);
        let was_open = PICKER_WAS_OPEN.swap(picker_open, Ordering::Relaxed);
        if picker_open && !selection.networks.is_empty() {
            // Picker just opened → start a fresh recolor session.
            if !was_open {
                ESTABLISHED.store(false, Ordering::Relaxed);
            }
            if ESTABLISHED.load(Ordering::Relaxed) {
                // Real-time: repaint the established fill slot in place.
                preview_fill_on_selection(committed, selection, *vector_fill_color);
            } else {
                // Establish ONCE — but only on the first ACTUAL colour change vs
                // the selection's current fill, so opening + closing the picker
                // without a change costs no undo step / no orphaned fill (#9).
                let current = selection
                    .networks
                    .first()
                    .and_then(|&i| committed.get(i))
                    .and_then(|a| {
                        a.network
                            .regions
                            .first()
                            .and_then(|r| r.fill)
                            .and_then(|fr| a.styles.fills.get(&fr))
                            .map(|f| f.color.to_srgb().0)
                    });
                if current != Some(*vector_fill_color) {
                    // First real change → one logged, undoable establish; later
                    // frames preview in place.
                    crate::input_dispatch::vector_undo::checkpoint(undo, redo, committed);
                    apply_fill_to_selection(committed, selection, *vector_fill_color);
                    ESTABLISHED.store(true, Ordering::Relaxed);
                }
            }
        }
    }

    // ── Load the SELECTED shape's current fill into the picker (Enio: "the
    //    color picker should assume the shape's current colour"). Only while the
    //    picker is CLOSED — once it's open, the live picked colour drives instead
    //    (and recolors the shape via the apply block above). So selecting a shape
    //    loads its colour; opening the picker then starts from there.
    if !picker_open
        && let Some(&idx) = selection.networks.first()
        && let Some(asset) = committed.get(idx)
        && let Some(region) = asset.network.regions.first()
        && let Some(fref) = region.fill
        && let Some(fill) = asset.styles.fills.get(&fref)
    {
        let rgba = fill.color.to_srgb().0;
        *vector_fill_color = rgba;
        hero.store
            .set_widget_color(ph2d_editor::ids::VECTOR_INSPECTOR_FILL_SWATCH, rgba);
    }

    // ── Publish the current fill color so the panel paints the swatch ─────
    #[cfg(feature = "panel-vector-inspector")]
    ph2d_panel_vector_inspector::set_current_fill(*vector_fill_color);

    // ── Inject the current fill colour into the active create-tool (#6) ───
    // The picker shows for select/direct/shape, but the chosen colour persists
    // in vector_fill_color across tool switches. Without this a shape/path
    // committed after picking still carried the seeded blue default. The setters
    // were tested but had ZERO shell callers. (Pen has no swatch yet but honours
    // the persisted colour.)
    {
        let fill = ph2d_vector_doc::srgb8_to_oklch(*vector_fill_color);
        if let Some(t) = tools.active_mut() {
            let any = t.as_any_mut();
            if let Some(s) = any.downcast_mut::<ph2d_tool_vector_shape::VectorShapeTool>() {
                s.set_default_fill(fill);
                s.set_default_stroke(fill, ph2d_tool_vector_shape::DEFAULT_STROKE_WIDTH_PX);
            } else if let Some(p) = any.downcast_mut::<ph2d_tool_vector_pen::VectorPenTool>() {
                p.set_default_fill(fill);
            } else if let Some(p) = any.downcast_mut::<ph2d_tool_vector_pencil::VectorPencilTool>() {
                p.set_default_stroke(fill, ph2d_tool_vector_pencil::DEFAULT_STROKE_WIDTH_PX);
            }
        }
    }

    // ── Shape-kind picker (§4.2) — the on-screen replacement for hotkeys 1-5.
    // Mirror of the fill read-back: drain a pending option click into the active
    // Shape tool (`set_kind`) and publish its current kind so the panel paints
    // the selected option. Only meaningful while `vector_shape` is active; the
    // downcast yields `None` otherwise (panel hides the row). ────────────────
    #[cfg(feature = "panel-vector-inspector")]
    {
        use ph2d_tool_vector_shape::{ShapeKind, VectorShapeTool};
        if let Some(shape) = tools
            .active_mut()
            .and_then(|t| t.as_any_mut().downcast_mut::<VectorShapeTool>())
        {
            if let Some(index) = ph2d_panel_vector_inspector::take_pending_shape_selection()
                && let Some(&kind) = ShapeKind::ALL.get(index as usize)
            {
                shape.set_kind(kind);
            }
            let idx = ShapeKind::ALL
                .iter()
                .position(|&k| k == shape.kind())
                .unwrap_or(0) as u8;
            ph2d_panel_vector_inspector::set_current_shape(true, idx);
        } else {
            ph2d_panel_vector_inspector::set_current_shape(false, 0);
        }
    }
}

#[cfg(all(test, feature = "panel-vector-inspector"))]
mod tests {
    use ph2d_tool_vector_shape::ShapeKind;

    /// The inspector's Shape-kind picker must expose exactly one option per
    /// `ShapeKind` variant — the panel is decoupled from the tool crate, so this
    /// cross-crate count is the only thing that catches a drift (e.g. the tool
    /// gains a 6th kind but the panel still paints 5). The drain
    /// (`ShapeKind::ALL.get(index)`) degrades safely on mismatch, but a stale
    /// picker would silently hide the new kind — this fails the build instead.
    #[test]
    fn shape_picker_exposes_one_option_per_kind() {
        assert_eq!(
            ph2d_panel_vector_inspector::ids::SHAPE_OPTION_IDS.len(),
            ShapeKind::ALL.len(),
            "add the new option id + label in ph2d-panel-vector-inspector::ids when \
             ShapeKind grows (and the matching VECTOR_INSPECTOR_SHAPE_* id in editor-core)"
        );
    }
}
