//! Panel → tool. Every arm derives from [`crate::rows`] — a row that exists
//! is a row that dispatches. Everything forwards through the frozen generic
//! `ToolPanelEvent` channel; the painter's `route_brush_wetpaint_event`
//! resolves the dynamic id family on the other side.

use ph2d_editor_core::action_bus::EditorAction;
use ph2d_editor_core::ids;
use ph2d_editor_core::interaction::WidgetEvent;
use ph2d_editor_core::panel::{EventOutcome, PanelHostInternal, seam_reset_button};
use ph2d_editor_core::tool::PanelEvent;

use crate::rows;

fn forward(host: &mut dyn PanelHostInternal, ev: PanelEvent) {
    host.bus_mut().push(EditorAction::ToolPanelEvent(ev));
}

pub(crate) fn apply_event(
    _state: &mut crate::state::WetTuningPanelState,
    host: &mut dyn PanelHostInternal,
    ev: WidgetEvent,
) -> EventOutcome {
    let consumed = match ev {
        // A slider moved: read the track the drag left behind, map it to the
        // row's REAL value, forward. The chip edit mirrors onto its linked
        // slider (which fires its own ValueChanged) — swallow the chip's, or
        // one edit notifies twice.
        WidgetEvent::ValueChanged(id) => {
            if let Some(row) = rows::row_for(id) {
                if id == row.chip {
                    let v = host.store().number_value(id).unwrap_or(row.default);
                    forward(host, PanelEvent::SetValue(row.chip, v));
                } else {
                    let track = host.store().slider(id).map(|(_, v)| v).unwrap_or(0.5);
                    forward(host, PanelEvent::SetValue(row.slider, row.value_of(track)));
                }
                true
            } else {
                false
            }
        }
        // Section headers fold (panel-local view state, never forwarded).
        WidgetEvent::Click(id)
            if rows::SECTIONS.iter().any(|s| s.header == id)
                || id == ids::WET_TUNING_GROUP_HEADERS[5] =>
        {
            seam_reset_button(host, id);
            let collapsed = host.store().is_collapsed(id);
            host.store_mut().set_collapsed(id, !collapsed);
            true
        }
        // Per-knob reset / group reset / the PAPER eye / the K–M checkboxes:
        // authored PAINTER state — forward the click, the tool routes it.
        WidgetEvent::Click(id)
            if rows::row_for(id).is_some_and(|r| r.reset == id)
                || rows::SECTIONS.iter().any(|s| s.reset == id)
                || id == ids::WET_TUNING_PAPER_EYE
                || id == ids::WET_TUNING_KM_MIXING
                || id == ids::WET_TUNING_KM_GLAZE =>
        {
            seam_reset_button(host, id);
            forward(host, PanelEvent::Click(id));
            true
        }
        // Close = the SAME Tuning toggle the basic section owns (visibility
        // is the tool's authored fact; the bridge mirrors it every frame —
        // a panel-local hide would fight the bridge and lose).
        WidgetEvent::Click(id) if id == ids::WET_TUNING_CLOSE => {
            seam_reset_button(host, id);
            forward(host, PanelEvent::Click(ids::PAINTER_WETPAINT_TUNING));
            true
        }
        _ => false,
    };
    EventOutcome::from_bool(consumed)
}
