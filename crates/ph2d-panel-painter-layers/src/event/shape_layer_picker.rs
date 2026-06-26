//! Per-layer-colour row handlers (multi-layer Shape): classify a factory id as a layer's colour
//! checkbox or its swatch, and open/seed the shared colour picker for a swatch (mirror of the brush
//! colour thumb + the ramp swatch). The checkbox click forwards as a plain `Click` (the tool toggles
//! the layer); the swatch toggles the picker, seeded from that layer's colour in the brush snapshot.

use crate::state;
use ph2d_a11y::NodeId;
use ph2d_editor_core::ids as core_ids;
use ph2d_editor_core::panel::PanelHostInternal;
use ph2d_tool_painter::MAX_SHAPE_LAYERS;

/// What a per-layer-colour factory id is.
pub(super) enum LayerWidget {
    /// The "Layer i+1 Color" checkbox (forward a `Click`; the tool toggles that layer's colour use).
    Check,
    /// The colour swatch for layer `i` (toggle the shared picker, seeded with the layer colour).
    Swatch(u8),
}

/// Classify `id` as one of the per-layer-colour row widgets, if it is one — a bounded scan over the cap.
pub(super) fn classify(id: NodeId) -> Option<LayerWidget> {
    for i in 0..MAX_SHAPE_LAYERS as u8 {
        if id == core_ids::painter_shape_layer_color_check_id(i) {
            return Some(LayerWidget::Check);
        }
        if id == core_ids::painter_shape_layer_color_swatch_id(i) {
            return Some(LayerWidget::Swatch(i));
        }
    }
    None
}

/// Layer `i`'s swatch was clicked → toggle the shared picker targeting it, seeded with the layer colour.
pub(super) fn on_swatch_click(host: &mut dyn PanelHostInternal, id: NodeId, i: u8) {
    if host.store().picker_target() == Some(id) {
        host.store_mut().set_picker_target(None);
        return;
    }
    let enc = |c: f32| (c.clamp(0.0, 1.0) * 255.0 + 0.5) as u8; // LITERAL-PX-OK: sRGB 8-bit normalize
    let rgba = state::current_brush()
        .map(|b| {
            let c = b.shape_layer_color[i as usize];
            [enc(c[0]), enc(c[1]), enc(c[2]), 255u8]
        })
        .unwrap_or([0, 0, 0, 255]);
    let store = host.store_mut();
    store.set_blender_value(
        core_ids::INSP_BLENDER_PICKER,
        ph2d_tokens::ColorValue::from_rgba8(rgba[0], rgba[1], rgba[2], rgba[3]),
    );
    store.set_widget_color(id, rgba);
    store.set_picker_target(Some(id));
}
