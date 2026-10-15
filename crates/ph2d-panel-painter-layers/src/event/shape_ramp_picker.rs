//! Shape **Colour Ramp** interaction handlers (the colour twin of `ramp_picker`), for BOTH ramps of the
//! Shape Color section — the **Texture Color** (the Shape ramp's ids) and the **Falloff Color**
//! (prompt 38): the colour-box picker open/seed and the `ValueChanged` reactions for the bar-stop drag +
//! the editable index / position chips. The selection is a stable stop id (so it follows a stop dragged
//! past a neighbour); the index chip resolves a typed sorted-index back to that id, the position chip
//! moves it. The colour itself is edited via the shared picker → the ramp's swatch id.

use crate::state;
use ph2d_a11y::NodeId;
use ph2d_editor_core::action_bus::EditorAction;
use ph2d_editor_core::ids as core_ids;
use ph2d_editor_core::panel::PanelHostInternal;
use ph2d_editor_core::tool::PanelEvent;
use ph2d_tool_painter::{BrushSettings, ids};

/// One of the two Shape ramps: its ids, its selection and where the snapshot carries its stops.
struct Rampa {
    swatch: NodeId,
    edit: NodeId,
    stop_index: NodeId,
    stop_pos: NodeId,
    selected: fn() -> u8,
    select: fn(u8),
    stops: fn(&BrushSettings) -> &[[f32; 6]],
}

const TEXTURA: Rampa = Rampa {
    swatch: ids::PAINTER_SHAPE_RAMP_SWATCH,
    edit: ids::PAINTER_SHAPE_RAMP_EDIT,
    stop_index: ids::PAINTER_SHAPE_RAMP_STOP_INDEX,
    stop_pos: ids::PAINTER_SHAPE_RAMP_STOP_POS,
    selected: state::selected_shape_ramp_stop,
    select: state::set_selected_shape_ramp_stop,
    stops: |b| {
        &b.shape_color_ramp_stops
            [..(b.shape_color_ramp_stop_count as usize).min(b.shape_color_ramp_stops.len())]
    },
};

const QUEDA: Rampa = Rampa {
    swatch: ids::PAINTER_SHAPE_FALLOFF_RAMP_SWATCH,
    edit: ids::PAINTER_SHAPE_FALLOFF_RAMP_EDIT,
    stop_index: ids::PAINTER_SHAPE_FALLOFF_RAMP_STOP_INDEX,
    stop_pos: ids::PAINTER_SHAPE_FALLOFF_RAMP_STOP_POS,
    selected: crate::state_shape_cor::selected_falloff_stop,
    select: crate::state_shape_cor::set_selected_falloff_stop,
    stops: |b| {
        &b.shape_falloff_ramp_stops
            [..(b.shape_falloff_ramp_stop_count as usize).min(b.shape_falloff_ramp_stops.len())]
    },
};

/// The ramp a widget id belongs to.
fn rampa_de(id: NodeId) -> Option<&'static Rampa> {
    [&TEXTURA, &QUEDA]
        .into_iter()
        .find(|r| [r.swatch, r.edit, r.stop_index, r.stop_pos].contains(&id))
}

/// Whether `id` is one of the two Shape ramps' colour boxes.
pub(super) fn is_swatch(id: NodeId) -> bool {
    id == TEXTURA.swatch || id == QUEDA.swatch
}

/// A Shape ramp's colour box was clicked → toggle the shared picker targeting it, seeded with the
/// selected stop's colour.
pub(super) fn on_swatch_click(host: &mut dyn PanelHostInternal, id: NodeId) {
    let Some(r) = rampa_de(id) else {
        return;
    };
    let store = host.store_mut();
    if store.picker_target() == Some(id) {
        store.set_picker_target(None);
    } else {
        let rgba = selected_stop_seed(r);
        store.set_blender_value(
            core_ids::INSP_BLENDER_PICKER,
            ph2d_tokens::ColorValue::from_rgba8(rgba[0], rgba[1], rgba[2], rgba[3]),
        );
        store.set_widget_color(id, rgba);
        store.set_picker_target(Some(id));
    }
}

/// Route a Shape-ramp `ValueChanged`: the bar-stop drag, or a commit on the editable index / position
/// chip. (Called by `value_forward.rs` for the two ramps' value ids only.)
pub(super) fn on_shape_ramp_value_changed(host: &mut dyn PanelHostInternal, id: NodeId) {
    let Some(r) = rampa_de(id) else {
        return;
    };
    if id == r.edit {
        // Bar-stop drag: the `CurvePoint` stashed `(_, _, stop_id, x, _)` → select it + forward `id:x`.
        let edit = r.edit;
        if let Some((_p, _c, stop_id, x, _y)) =
            host.store_mut().take_curve_point_drag_if(|p| p == edit)
        {
            (r.select)(stop_id);
            forward(host, r.edit, format!("{stop_id}:{x}"));
        }
    } else if id == r.stop_index {
        on_index_commit(host, r);
    } else if id == r.stop_pos
        && let Some(v) = host.store().number_value(r.stop_pos)
    {
        let sel = (r.selected)();
        forward(
            host,
            r.edit,
            format!("{sel}:{}", (v as f32).clamp(0.0, 1.0)),
        );
    }
}

/// The index selector committed: select the stop at that sorted index (clamped), tracked by stable id.
fn on_index_commit(host: &mut dyn PanelHostInternal, r: &Rampa) {
    let Some(v) = host.store().number_value(r.stop_index) else {
        return;
    };
    let Some(b) = state::current_brush() else {
        return;
    };
    let stops = (r.stops)(&b);
    if stops.is_empty() {
        return;
    }
    let target = (v.round() as i64).clamp(0, stops.len() as i64 - 1) as usize; // CLAMP-OK: integer (i64) bounds, no NaN; non-empty guarded above
    (r.select)(stops[target][5] as u8);
}

/// The selected stop's colour (sRGB bytes) — the picker's seed (found by stable id, not array index).
fn selected_stop_seed(r: &Rampa) -> [u8; 4] {
    let enc = |c: f32| (c.clamp(0.0, 1.0) * 255.0 + 0.5) as u8; // LITERAL-PX-OK: sRGB 8-bit normalize
    let sel_id = (r.selected)();
    state::current_brush()
        .and_then(|b| (r.stops)(&b).iter().find(|s| s[5] as u8 == sel_id).copied())
        .map(|s| [enc(s[1]), enc(s[2]), enc(s[3]), enc(s[4])])
        .unwrap_or([0, 0, 0, 255])
}

fn forward(host: &mut dyn PanelHostInternal, id: NodeId, value: String) {
    host.bus_mut()
        .push(EditorAction::ToolPanelEvent(PanelEvent::SelectOption(
            id, value,
        )));
}
