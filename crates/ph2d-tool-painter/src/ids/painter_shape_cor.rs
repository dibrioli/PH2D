//! The **Shape Color** section's two LAYERS (prompt 38, dono 15/10): the **Texture Color** (the Shape
//! ramp of [`super::painter_shape`], indexed by the Shape's texture value) and the **Falloff Color** (a
//! second ramp, indexed by the dab coverage — the pre-prompt-38 law), their order and the Blend between
//! them. The Falloff ramp's editor is a full twin of the Shape ramp's (same widget, its own ids).

use ph2d_a11y::NodeId;
use ph2d_tool_registry::{hash_node_id, hash_node_id_runtime};

/// **Texture Color** layer checkbox. `Click` → `toggle_shape_texture_color`.
pub const PAINTER_SHAPE_TEXTURE_COLOR: NodeId = hash_node_id("painter_brush.shape_texture_color");

/// **Falloff Color** layer checkbox. `Click` → `toggle_shape_falloff_color`.
pub const PAINTER_SHAPE_FALLOFF_COLOR: NodeId = hash_node_id("painter_brush.shape_falloff_color");

/// Swap which layer is on top (the first). `Click` → `swap_shape_color_layers`.
pub const PAINTER_SHAPE_COLOR_SWAP: NodeId = hash_node_id("painter_brush.shape_color_swap");

/// The **Blend** mode of the top layer over the bottom one (`BrushBlend`). `SelectOption` →
/// `set_shape_color_blend`.
pub const PAINTER_SHAPE_COLOR_BLEND: NodeId = hash_node_id("painter_brush.shape_color_blend");

/// The top layer's opacity (**Mix**, `0..1`). `SetValue` → `set_shape_color_mix`.
pub const PAINTER_SHAPE_COLOR_MIX: NodeId = hash_node_id("painter_brush.shape_color_mix");

/// Falloff ramp **colour mode** dropdown. `SelectOption` → `set_shape_falloff_ramp_mode`.
pub const PAINTER_SHAPE_FALLOFF_RAMP_MODE: NodeId =
    hash_node_id("painter_brush.shape_falloff_ramp_mode");

/// Falloff ramp **interpolation** dropdown. `SelectOption` → `set_shape_falloff_ramp_interp`.
pub const PAINTER_SHAPE_FALLOFF_RAMP_INTERP: NodeId =
    hash_node_id("painter_brush.shape_falloff_ramp_interp");

/// The Falloff ramp's gradient **bar** (the `CurvePoint` parent of its stop handles).
pub const PAINTER_SHAPE_FALLOFF_RAMP_EDIT: NodeId =
    hash_node_id("painter_brush.shape_falloff_ramp_edit");

/// "+" add a Falloff-ramp stop. `Click` → `shape_falloff_ramp_add_stop`.
pub const PAINTER_SHAPE_FALLOFF_RAMP_ADD: NodeId =
    hash_node_id("painter_brush.shape_falloff_ramp_add");

/// "−" remove the last Falloff-ramp stop. `Click` → `shape_falloff_ramp_remove_last_stop`.
pub const PAINTER_SHAPE_FALLOFF_RAMP_REMOVE: NodeId =
    hash_node_id("painter_brush.shape_falloff_ramp_remove");

/// **Invert** the Falloff ramp. `Click` → `shape_falloff_ramp_invert`.
pub const PAINTER_SHAPE_FALLOFF_RAMP_INVERT: NodeId =
    hash_node_id("painter_brush.shape_falloff_ramp_invert");

/// The selected Falloff-ramp stop's **colour swatch** — opens the colour picker.
pub const PAINTER_SHAPE_FALLOFF_RAMP_SWATCH: NodeId =
    hash_node_id("painter_brush.shape_falloff_ramp_swatch");

/// Selected Falloff-ramp stop **index** chip (`NumberInput`).
pub const PAINTER_SHAPE_FALLOFF_RAMP_STOP_INDEX: NodeId =
    hash_node_id("painter_brush.shape_falloff_ramp_stop_index");

/// Selected Falloff-ramp stop **position** chip (`NumberInput`, `0..1`).
pub const PAINTER_SHAPE_FALLOFF_RAMP_STOP_POS: NodeId =
    hash_node_id("painter_brush.shape_falloff_ramp_stop_pos");

/// The layers' **Click** ids (the two checkboxes, the swap, the Falloff ramp's `+ − I`) — forwarded as
/// `PanelEvent::Click` by the panel's `event.rs` (one membership check).
pub const PAINTER_SHAPE_COLOR_BUTTONS: [NodeId; 6] = [
    PAINTER_SHAPE_TEXTURE_COLOR,
    PAINTER_SHAPE_FALLOFF_COLOR,
    PAINTER_SHAPE_COLOR_SWAP,
    PAINTER_SHAPE_FALLOFF_RAMP_ADD,
    PAINTER_SHAPE_FALLOFF_RAMP_REMOVE,
    PAINTER_SHAPE_FALLOFF_RAMP_INVERT,
];

/// The Falloff ramp's **ValueChanged** ids (bar-stop drag + index / position chips).
pub const PAINTER_SHAPE_FALLOFF_RAMP_VALUE_IDS: [NodeId; 3] = [
    PAINTER_SHAPE_FALLOFF_RAMP_EDIT,
    PAINTER_SHAPE_FALLOFF_RAMP_STOP_INDEX,
    PAINTER_SHAPE_FALLOFF_RAMP_STOP_POS,
];

/// Stable [`NodeId`] for the draggable handle of Falloff-ramp stop `i`.
#[must_use]
pub fn painter_shape_falloff_ramp_handle_id(i: u8) -> NodeId {
    hash_node_id_runtime(&format!("painter_brush.shapefalloffhandle.{i}"))
}

/// Stable [`NodeId`] for Falloff-ramp colour-mode option `m` in the open Mode dropdown popover.
#[must_use]
pub fn painter_shape_falloff_ramp_mode_option_id(m: u8) -> NodeId {
    hash_node_id_runtime(&format!("painter_brush.shapefalloffmodeopt.{m}"))
}

/// Stable [`NodeId`] for Falloff-ramp interpolation option `i` in the open dropdown popover.
#[must_use]
pub fn painter_shape_falloff_ramp_interp_option_id(i: u8) -> NodeId {
    hash_node_id_runtime(&format!("painter_brush.shapefalloffinterpopt.{i}"))
}

/// Stable [`NodeId`] for Blend option `m` (a `BrushBlend` discriminant) in the open popover.
#[must_use]
pub fn painter_shape_color_blend_option_id(m: u8) -> NodeId {
    hash_node_id_runtime(&format!("painter_brush.shapecolorblendopt.{m}"))
}
