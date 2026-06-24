//! Color Ramp interaction handlers split from `event.rs` for the LOC cap: the colour-box picker
//! open/seed, and the `ValueChanged` reactions for the bar-stop drag + the editable index / position
//! chips. The selection is a **stable stop id** (so it follows a stop dragged past a neighbour); the
//! index chip resolves a typed sorted-index back to that id, the position chip moves the selected id.

use crate::state;
use ph2d_a11y::NodeId;
use ph2d_editor_core::action_bus::EditorAction;
use ph2d_editor_core::ids as core_ids;
use ph2d_editor_core::panel::PanelHostInternal;
use ph2d_editor_core::tool::PanelEvent;

/// The Color Ramp colour box was clicked → toggle the shared picker targeting it, seeded with the
/// selected stop's colour.
pub(super) fn on_swatch_click(host: &mut dyn PanelHostInternal) {
    let id = core_ids::PAINTER_BRUSH_TEXTURE_RAMP_SWATCH;
    let store = host.store_mut();
    if store.picker_target() == Some(id) {
        store.set_picker_target(None);
    } else {
        let rgba = selected_stop_seed();
        store.set_blender_value(
            core_ids::INSP_BLENDER_PICKER,
            ph2d_tokens::ColorValue::from_rgba8(rgba[0], rgba[1], rgba[2], rgba[3]),
        );
        store.set_widget_color(id, rgba);
        store.set_picker_target(Some(id));
    }
}

/// Route a Color Ramp `ValueChanged`: the bar-stop drag, or a commit on the editable index / position
/// chip. (Called by `event.rs` for the three ramp ids only.)
pub(super) fn on_ramp_value_changed(host: &mut dyn PanelHostInternal, id: NodeId) {
    if id == core_ids::PAINTER_BRUSH_TEXTURE_RAMP_EDIT {
        // Bar-stop drag: the `CurvePoint` stashed `(_, _, stop_id, x, _)` → select it + forward `id:x`.
        if let Some((_p, _c, stop_id, x, _y)) = host.store_mut().take_curve_point_drag() {
            state::set_selected_ramp_stop(stop_id);
            host.bus_mut()
                .push(EditorAction::ToolPanelEvent(PanelEvent::SelectOption(
                    core_ids::PAINTER_BRUSH_TEXTURE_RAMP_EDIT,
                    format!("{stop_id}:{x}"),
                )));
        }
    } else if id == core_ids::PAINTER_BRUSH_TEXTURE_RAMP_STOP_INDEX {
        on_index_commit(host);
    } else if id == core_ids::PAINTER_BRUSH_TEXTURE_RAMP_STOP_POS {
        on_position_commit(host);
    }
}

/// The index selector committed: select the stop at that sorted index (clamped), tracked by its
/// stable id so the selection survives later re-sorts. Panel-local — no tool edit.
fn on_index_commit(host: &mut dyn PanelHostInternal) {
    let Some(v) = host
        .store()
        .number_value(core_ids::PAINTER_BRUSH_TEXTURE_RAMP_STOP_INDEX)
    else {
        return;
    };
    // When a Texture layer is being edited, resolve the index against ITS ramp, not the brush's.
    let Some(b) = crate::paint_texture::active_texture_ramp_view().or_else(state::current_brush)
    else {
        return;
    };
    let count = (b.texture_ramp_stop_count as usize).min(b.texture_ramp_stops.len());
    if count == 0 {
        return;
    }
    let target = (v.round() as i64).clamp(0, count as i64 - 1) as usize;
    state::set_selected_ramp_stop(b.texture_ramp_stops[target][5] as u8);
}

/// The position chip committed: move the selected stop (by stable id) to the typed `0..1` position.
fn on_position_commit(host: &mut dyn PanelHostInternal) {
    let Some(v) = host
        .store()
        .number_value(core_ids::PAINTER_BRUSH_TEXTURE_RAMP_STOP_POS)
    else {
        return;
    };
    let pos = (v as f32).clamp(0.0, 1.0);
    let sel_id = state::selected_ramp_stop();
    host.bus_mut()
        .push(EditorAction::ToolPanelEvent(PanelEvent::SelectOption(
            core_ids::PAINTER_BRUSH_TEXTURE_RAMP_EDIT,
            format!("{sel_id}:{pos}"),
        )));
}

/// The selected stop's colour (sRGB bytes) — the picker's seed. Reads the **active Texture layer's**
/// ramp when one is being edited (else the brush snapshot), so the picker opens on the layer's own
/// stop colour and the readback's equality guard short-circuits instead of overwriting the stop with
/// the brush colour on open. The selection is a stable id (found by id, not array index).
fn selected_stop_seed() -> [u8; 4] {
    let enc = |c: f32| (c.clamp(0.0, 1.0) * 255.0 + 0.5) as u8;
    let sel_id = state::selected_ramp_stop();
    crate::paint_texture::active_texture_ramp_view()
        .or_else(state::current_brush)
        .and_then(|b| {
            let count = (b.texture_ramp_stop_count as usize).min(b.texture_ramp_stops.len());
            b.texture_ramp_stops[..count]
                .iter()
                .find(|s| s[5] as u8 == sel_id)
                .copied()
        })
        .map(|s| [enc(s[1]), enc(s[2]), enc(s[3]), enc(s[4])])
        .unwrap_or([0, 0, 0, 255])
}
