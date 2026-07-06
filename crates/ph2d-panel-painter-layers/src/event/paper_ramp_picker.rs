//! **Paper Colors** ramp interaction handlers (mirror of [`super::shape_ramp_picker`], on the Paper
//! ramp ids): the colour-box picker open/seed + the `ValueChanged` reactions for the bar-stop drag +
//! the editable index / position chips. Selection is a stable stop id.

use crate::state;
use ph2d_a11y::NodeId;
use ph2d_editor_core::action_bus::EditorAction;
use ph2d_editor_core::ids as core_ids;
use ph2d_editor_core::panel::PanelHostInternal;
use ph2d_editor_core::tool::PanelEvent;

/// The Paper ramp colour box was clicked → toggle the shared picker, seeded with the selected stop's colour.
pub(super) fn on_swatch_click(host: &mut dyn PanelHostInternal) {
    let id = core_ids::PAINTER_PAPER_RAMP_SWATCH;
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

/// Route a Paper-ramp `ValueChanged`: the bar-stop drag, or a commit on the editable index / position chip.
pub(super) fn on_paper_ramp_value_changed(host: &mut dyn PanelHostInternal, id: NodeId) {
    if id == core_ids::PAINTER_PAPER_RAMP_EDIT {
        if let Some((_p, _c, stop_id, x, _y)) = host.store_mut().take_curve_point_drag() {
            state::set_selected_paper_ramp_stop(stop_id);
            forward(
                host,
                core_ids::PAINTER_PAPER_RAMP_EDIT,
                format!("{stop_id}:{x}"),
            );
        }
    } else if id == core_ids::PAINTER_PAPER_RAMP_STOP_INDEX {
        on_index_commit(host);
    } else if id == core_ids::PAINTER_PAPER_RAMP_STOP_POS
        && let Some(v) = host
            .store()
            .number_value(core_ids::PAINTER_PAPER_RAMP_STOP_POS)
    {
        let sel = state::selected_paper_ramp_stop();
        forward(
            host,
            core_ids::PAINTER_PAPER_RAMP_EDIT,
            format!("{sel}:{}", (v as f32).clamp(0.0, 1.0)),
        );
    }
}

/// The index selector committed: select the stop at that sorted index (clamped), tracked by stable id.
fn on_index_commit(host: &mut dyn PanelHostInternal) {
    let Some(v) = host
        .store()
        .number_value(core_ids::PAINTER_PAPER_RAMP_STOP_INDEX)
    else {
        return;
    };
    let Some(b) = state::current_brush() else {
        return;
    };
    let count = (b.paper_color_ramp_stop_count as usize).min(b.paper_color_ramp_stops.len());
    if count == 0 {
        return;
    }
    let target = (v.round() as i64).clamp(0, count as i64 - 1) as usize; // CLAMP-OK: integer bounds, count>=1
    state::set_selected_paper_ramp_stop(b.paper_color_ramp_stops[target][5] as u8);
}

/// The selected stop's colour (sRGB bytes) — the picker's seed (found by stable id, not array index).
fn selected_stop_seed() -> [u8; 4] {
    let enc = |c: f32| (c.clamp(0.0, 1.0) * 255.0 + 0.5) as u8; // LITERAL-PX-OK: sRGB 8-bit normalize
    let sel_id = state::selected_paper_ramp_stop();
    state::current_brush()
        .and_then(|b| {
            let count =
                (b.paper_color_ramp_stop_count as usize).min(b.paper_color_ramp_stops.len());
            b.paper_color_ramp_stops[..count]
                .iter()
                .find(|s| s[5] as u8 == sel_id)
                .copied()
        })
        .map(|s| [enc(s[1]), enc(s[2]), enc(s[3]), enc(s[4])])
        .unwrap_or([0, 0, 0, 255])
}

fn forward(host: &mut dyn PanelHostInternal, id: NodeId, value: String) {
    host.bus_mut()
        .push(EditorAction::ToolPanelEvent(PanelEvent::SelectOption(
            id, value,
        )));
}
