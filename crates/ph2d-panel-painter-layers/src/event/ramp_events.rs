//! Dispatch helpers for the three colour-ramp editors (Grain / Shape / Paper) — the swatch-box click and
//! the bar-stop / index / position `ValueChanged`. Consolidated here so `event.rs` carries one arm each
//! instead of three (keeping that file under the LOC cap).

use super::{paper_ramp_picker, ramp_picker, shape_ramp_picker};
use ph2d_a11y::NodeId;
use ph2d_editor_core::ids as core_ids;
use ph2d_editor_core::panel::PanelHostInternal;

/// Whether `id` is a ramp colour-box swatch that opens the shared picker (Shape / Paper — the Grain
/// swatch is handled by the generic dropdown path).
pub(super) fn is_ramp_swatch(id: NodeId) -> bool {
    id == core_ids::PAINTER_SHAPE_RAMP_SWATCH || id == core_ids::PAINTER_PAPER_RAMP_SWATCH
}

/// Open the shared picker for the clicked ramp swatch.
pub(super) fn on_ramp_swatch_click(host: &mut dyn PanelHostInternal, id: NodeId) {
    if id == core_ids::PAINTER_SHAPE_RAMP_SWATCH {
        shape_ramp_picker::on_swatch_click(host);
    } else {
        paper_ramp_picker::on_swatch_click(host);
    }
}

/// Whether `id` is a ramp value widget (bar-stop drag / editable index / position) for any of the three ramps.
pub(super) fn is_ramp_value_id(id: NodeId) -> bool {
    core_ids::PAINTER_BRUSH_TEXTURE_RAMP_VALUE_IDS.contains(&id)
        || core_ids::PAINTER_SHAPE_RAMP_VALUE_IDS.contains(&id)
        || core_ids::PAINTER_PAPER_RAMP_VALUE_IDS.contains(&id)
}

/// Route a ramp value `ValueChanged` to the owning ramp's picker.
pub(super) fn on_ramp_value_changed(host: &mut dyn PanelHostInternal, id: NodeId) {
    if core_ids::PAINTER_BRUSH_TEXTURE_RAMP_VALUE_IDS.contains(&id) {
        ramp_picker::on_ramp_value_changed(host, id);
    } else if core_ids::PAINTER_SHAPE_RAMP_VALUE_IDS.contains(&id) {
        shape_ramp_picker::on_shape_ramp_value_changed(host, id);
    } else {
        paper_ramp_picker::on_paper_ramp_value_changed(host, id);
    }
}
