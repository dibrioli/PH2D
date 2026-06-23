//! Color Ramp per-stop colour-picker open handler. Split from `event.rs` for the LOC cap; mirrors
//! the brush colour-thumb open (toggle the shared `INSP_BLENDER_PICKER` targeting the stop swatch,
//! seeded with the stop's current colour). The live picker value is read back + forwarded by
//! `paint_texture_ramp::ramp_color_readback`.

use super::decode::decode_texture_ramp_stop;
use crate::state;
use ph2d_a11y::NodeId;
use ph2d_editor_core::ids as core_ids;
use ph2d_editor_core::panel::PanelHostInternal;

/// A ramp stop swatch was clicked → toggle the shared picker targeting that stop, seeding it with the
/// stop's current colour. Returns whether it was a stop swatch (handled).
pub(super) fn on_stop_click(host: &mut dyn PanelHostInternal, id: NodeId) -> bool {
    let Some(i) = decode_texture_ramp_stop(id) else {
        return false;
    };
    let store = host.store_mut();
    if store.picker_target() == Some(id) {
        store.set_picker_target(None);
    } else {
        let rgba = seed(i);
        store.set_blender_value(
            core_ids::INSP_BLENDER_PICKER,
            ph2d_tokens::ColorValue::from_rgba8(rgba[0], rgba[1], rgba[2], rgba[3]),
        );
        store.set_widget_color(id, rgba);
        store.set_picker_target(Some(id));
    }
    true
}

/// The stop's current colour (sRGB bytes) from the published brush snapshot — the picker's seed.
fn seed(i: u8) -> [u8; 4] {
    let enc = |c: f32| (c.clamp(0.0, 1.0) * 255.0 + 0.5) as u8;
    state::current_brush()
        .and_then(|b| b.texture_ramp_stops.get(i as usize).copied())
        .map(|s| [enc(s[1]), enc(s[2]), enc(s[3]), enc(s[4])])
        .unwrap_or([0, 0, 0, 255])
}
