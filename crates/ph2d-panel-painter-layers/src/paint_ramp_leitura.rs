//! The colour box's live **readback** of a Colour Ramp editor ([`crate::paint_ramp_widget`]): while the
//! shared picker targets the selected stop's box, its colour is forwarded to the tool. Split from the
//! widget for the panel LOC cap when the Shape Color section grew its second ramp (prompt 38).

use crate::paint_ramp_widget::{RampIds, RampView};
use ph2d_editor_core::action_bus::EditorAction;
use ph2d_editor_core::panel::PaintCtx;
use ph2d_editor_core::tool::PanelEvent;

/// When the picker targets this ramp's colour box, forward its live colour (incl. alpha) to the
/// selected stop (as `"id,r,g,b,a"`) once it differs. The tool's swatch decode sets that stop's colour.
pub(crate) fn ramp_color_readback(ctx: &mut PaintCtx, ids: &RampIds, view: RampView, sel: usize) {
    if ctx.host.store().picker_target() != Some(ids.swatch) {
        return;
    }
    let Some(picked) = ctx.host.store().widget_color(ids.swatch) else {
        return;
    };
    if sel >= view.stops.len() {
        return;
    }
    let s = view.stops[sel];
    let enc = |c: f32| (c.clamp(0.0, 1.0) * 255.0 + 0.5) as u8; // LITERAL-PX-OK: sRGB 8-bit normalize
    if [enc(s[1]), enc(s[2]), enc(s[3]), enc(s[4])] == picked {
        return;
    }
    let stop_id = s[5] as u8;
    ctx.host
        .bus_mut()
        .push(EditorAction::ToolPanelEvent(PanelEvent::SelectOption(
            ids.swatch,
            format!(
                "{stop_id},{},{},{},{}",
                picked[0], picked[1], picked[2], picked[3]
            ),
        )));
}
