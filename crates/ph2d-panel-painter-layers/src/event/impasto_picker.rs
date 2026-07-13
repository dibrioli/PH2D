//! The Impasto **colour swatches** → the shared OKLCH picker.
//!
//! Two swatches, one door, because they are the same gesture: the lamp's colour (what light falls on
//! the paint) and the paint's **Wax filter** (what the light picks up on its way THROUGH the paint).
//! The click toggles the picker targeting the swatch, seeded with the value it is showing; the
//! read-back rides `SelectOption` from `paint_impasto_rig` / `paint_impasto`.
//!
//! They arrived as two near-identical modules and were merged when the duplication pushed `event.rs`
//! over its LOC cap — which is the cap doing its job: it noticed the copy before a reader did.

use crate::state;
use ph2d_a11y::NodeId;
use ph2d_editor_core::ids as core_ids;
use ph2d_editor_core::panel::PanelHostInternal;

/// Whether `id` is one of the Impasto swatches this module owns.
pub(super) fn is_impasto_swatch(id: NodeId) -> bool {
    id == core_ids::PAINTER_IMPASTO_LIGHT_COLOR || id == core_ids::PAINTER_IMPASTO_WAX_COLOR
}

/// A swatch was clicked → toggle the shared picker targeting it, seeded with the colour it shows.
pub(super) fn on_swatch_click(host: &mut dyn PanelHostInternal, id: NodeId) {
    if host.store().picker_target() == Some(id) {
        host.store_mut().set_picker_target(None);
        return;
    }
    let enc = |c: f32| (c.clamp(0.0, 1.0) * 255.0 + 0.5) as u8; // LITERAL-PX-OK: sRGB 8-bit normalize
    // The fallbacks are not arbitrary: a lamp with no snapshot yet is WHITE (the default key), and an
    // unset Wax filter is WHITE because white means *the filter is open* — the light comes back wearing
    // the paint's own colour, which is the physics and is what the pass does with no filter at all.
    let rgba = state::current_brush()
        .map(|b| {
            let c = if id == core_ids::PAINTER_IMPASTO_WAX_COLOR {
                b.impasto_wax_color
            } else {
                b.impasto_rig.current().color
            };
            [enc(c[0]), enc(c[1]), enc(c[2]), 255u8]
        })
        .unwrap_or([255, 255, 255, 255]);
    let store = host.store_mut();
    store.set_blender_value(
        core_ids::INSP_BLENDER_PICKER,
        ph2d_tokens::ColorValue::from_rgba8(rgba[0], rgba[1], rgba[2], rgba[3]),
    );
    store.set_widget_color(id, rgba);
    store.set_picker_target(Some(id));
}
