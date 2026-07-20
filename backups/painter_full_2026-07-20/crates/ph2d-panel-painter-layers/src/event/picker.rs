//! Shared-picker toggling for the panel's colour swatches — the brush colour thumb + the
//! watercolor PAPER-colour thumb both open the shared Blender picker (`INSP_BLENDER_PICKER`),
//! seeded with their current colour; the per-frame read-backs (`paint_brush` /
//! `paint_watercolor`) forward the picked value to the tool. Split from `event.rs` (LOC cap).

use crate::state;
use ph2d_editor_core::ids as core_ids;
use ph2d_editor_core::panel::PanelHostInternal;

/// Toggle the shared Blender picker onto swatch `id` (or off, when it already owns it), seeded
/// with the swatch's current colour. `None` when `id` is not one of the picker-backed swatches.
pub(crate) fn try_toggle_shared_picker(
    host: &mut dyn PanelHostInternal,
    id: ph2d_a11y::NodeId,
) -> Option<bool> {
    let seed = if id == core_ids::PAINTER_COLOR_THUMB {
        brush_seed_rgba8()
    } else if id == core_ids::PAINTER_WATERCOLOR_PAPER_COLOR_THUMB {
        paper_seed_rgba8()
    } else {
        return None;
    };
    let store = host.store_mut();
    if store.picker_target() == Some(id) {
        store.set_picker_target(None);
    } else {
        store.set_blender_value(
            core_ids::INSP_BLENDER_PICKER,
            ph2d_tokens::ColorValue::from_rgba8(seed[0], seed[1], seed[2], seed[3]),
        );
        store.set_widget_color(id, seed);
        store.set_picker_target(Some(id));
    }
    Some(true)
}

/// The document paper colour as 8-bit RGBA (opaque), to seed the shared picker.
fn paper_seed_rgba8() -> [u8; 4] {
    let enc = |c: f32| (c.clamp(0.0, 1.0) * 255.0 + 0.5) as u8; // LITERAL-PX-OK: sRGB 8-bit normalize
    state::current_brush()
        .map(|b| {
            [
                enc(b.paper_color[0]),
                enc(b.paper_color[1]),
                enc(b.paper_color[2]),
                255,
            ]
        })
        .unwrap_or([255, 255, 255, 255])
}

/// The active brush colour as 8-bit RGBA (opaque), to seed the shared picker.
fn brush_seed_rgba8() -> [u8; 4] {
    let enc = |c: f32| (c.clamp(0.0, 1.0) * 255.0 + 0.5) as u8; // LITERAL-PX-OK: sRGB 8-bit normalize
    state::current_brush()
        .map(|b| [enc(b.color[0]), enc(b.color[1]), enc(b.color[2]), 255])
        .unwrap_or([0, 0, 0, 255])
}
