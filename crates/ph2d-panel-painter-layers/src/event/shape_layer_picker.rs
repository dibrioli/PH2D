//! Per-layer-colour row handlers (multi-layer Shape): classify a factory id as a layer's colour
//! checkbox or its swatch, and open/seed the shared colour picker for a swatch (mirror of the brush
//! colour thumb + the ramp swatch). The checkbox click forwards as a plain `Click` (the tool toggles
//! the layer); the swatch toggles the picker, seeded from that layer's colour in the brush snapshot.

use crate::state;
use ph2d_a11y::NodeId;
use ph2d_editor_core::action_bus::EditorAction;
use ph2d_editor_core::ids as core_ids;
use ph2d_editor_core::interaction::InteractiveState;
use ph2d_editor_core::panel::PanelHostInternal;
use ph2d_editor_core::tool::PanelEvent;
use ph2d_tool_painter::MAX_SHAPE_LAYERS;

/// What a per-layer-colour factory id is.
pub(super) enum LayerWidget {
    /// The "Layer i+1 Color" checkbox (forward a `Click`; the tool toggles that layer's custom colour).
    Check,
    /// The colour swatch for layer `i` (toggle the shared picker, seeded with the layer colour).
    Swatch(u8),
}

/// Classify `id` as one of the per-layer-colour row CLICK widgets, if it is one — a bounded scan over
/// the cap. (The "B" blend chip opens/closes via the generic Dropdown dispatch; its OPTION clicks are
/// classified by [`blend_option`]; the opacity box `ValueChanged` by [`opacity_index`].)
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

/// Classify `id` as a Shape-layer blend-popover OPTION → `(layer_index, mode)` — a bounded scan over the
/// per-layer × per-mode id space (only the open popover's options are hit-registered, so the scan is rare).
pub(super) fn blend_option(id: NodeId) -> Option<(u8, u8)> {
    for i in 0..MAX_SHAPE_LAYERS as u8 {
        for m in 0..ph2d_tool_painter::MAX_BLEND_MODES {
            if id == core_ids::painter_shape_layer_blend_option_id(i, m) {
                return Some((i, m));
            }
        }
    }
    None
}

/// A blend option `id` was picked → close its layer's "B" chip + forward `SelectOption(blend_id, mode)`
/// (the tool maps it to `set_brush_shape_layer_blend(i, mode)`). A no-op if `id` isn't a blend option.
pub(super) fn on_blend_option(host: &mut dyn PanelHostInternal, id: NodeId) {
    let Some((i, mode)) = blend_option(id) else {
        return;
    };
    let chip_id = core_ids::painter_shape_layer_blend_id(i);
    if let Some(InteractiveState::Dropdown {
        open,
        selected_index,
        ..
    }) = host.store_mut().get_mut(chip_id)
    {
        *open = false;
        *selected_index = Some(mode as usize);
    }
    host.bus_mut()
        .push(EditorAction::ToolPanelEvent(PanelEvent::SelectOption(
            chip_id,
            mode.to_string(),
        )));
}

/// Classify `id` as a Shape-layer **opacity** number box → its layer index, if it is one.
pub(super) fn opacity_index(id: NodeId) -> Option<u8> {
    (0..MAX_SHAPE_LAYERS as u8).find(|&i| id == core_ids::painter_shape_layer_opacity_id(i))
}

/// Forward the opacity box's scrubbed value as `SetValue` (the tool scales the layer's tip by it).
pub(super) fn forward_opacity(host: &mut dyn PanelHostInternal, id: NodeId) {
    let v = host.store().number_value(id).unwrap_or(0.0);
    host.bus_mut()
        .push(EditorAction::ToolPanelEvent(PanelEvent::SetValue(id, v)));
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
