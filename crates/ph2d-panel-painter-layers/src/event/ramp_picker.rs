//! Color Ramp colour-box picker open handler. Split from `event.rs` for the LOC cap; mirrors the
//! brush colour-thumb open (toggle the shared `INSP_BLENDER_PICKER` targeting the ramp colour box,
//! seeded with the **selected** stop's colour). The live value is read back + forwarded by
//! `paint_texture_ramp::ramp_color_readback`.

use crate::state;
use ph2d_editor_core::ids as core_ids;
use ph2d_editor_core::panel::PanelHostInternal;

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

/// The selected stop's colour (sRGB bytes) from the published brush snapshot — the picker's seed.
fn selected_stop_seed() -> [u8; 4] {
    let enc = |c: f32| (c.clamp(0.0, 1.0) * 255.0 + 0.5) as u8;
    let i = state::selected_ramp_stop() as usize;
    state::current_brush()
        .and_then(|b| b.texture_ramp_stops.get(i).copied())
        .map(|s| [enc(s[1]), enc(s[2]), enc(s[3]), enc(s[4])])
        .unwrap_or([0, 0, 0, 255])
}
