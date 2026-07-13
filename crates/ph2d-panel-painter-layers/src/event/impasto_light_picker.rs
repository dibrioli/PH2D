//! The Impasto lamp's **colour swatch** → the shared OKLCH picker. Mirror of `shape_layer_picker`'s
//! swatch half (and of the brush colour thumb, and the ramp swatch): the click toggles the picker
//! targeting the swatch, seeded with the SELECTED lamp's colour; the read-back rides `SelectOption` from
//! `paint_impasto_rig`.

use crate::state;
use ph2d_a11y::NodeId;
use ph2d_editor_core::ids as core_ids;
use ph2d_editor_core::panel::PanelHostInternal;

/// The lamp swatch was clicked → toggle the shared picker targeting it, seeded with the lamp's colour.
pub(super) fn on_swatch_click(host: &mut dyn PanelHostInternal, id: NodeId) {
    if host.store().picker_target() == Some(id) {
        host.store_mut().set_picker_target(None);
        return;
    }
    let enc = |c: f32| (c.clamp(0.0, 1.0) * 255.0 + 0.5) as u8; // LITERAL-PX-OK: sRGB 8-bit normalize
    let rgba = state::current_brush()
        .map(|b| {
            let c = b.impasto_rig.current().color;
            [enc(c[0]), enc(c[1]), enc(c[2]), 255u8]
        })
        .unwrap_or([255, 255, 255, 255]); // a lamp with no snapshot yet is white, like the default key
    let store = host.store_mut();
    store.set_blender_value(
        core_ids::INSP_BLENDER_PICKER,
        ph2d_tokens::ColorValue::from_rgba8(rgba[0], rgba[1], rgba[2], rgba[3]),
    );
    store.set_widget_color(id, rgba);
    store.set_picker_target(Some(id));
}
