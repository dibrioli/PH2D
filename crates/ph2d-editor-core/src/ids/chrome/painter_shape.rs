//! Brush **Shape** section NodeIds (the silhouette tip; Procreate "Shape"). The Shape section hosts
//! the Falloff (the procedural silhouette, default) plus — once a Shape **image** is assigned — the
//! image preview and its rotation controls (the falloff then goes inactive). Also the **Grain Depth**
//! slider (which lives in the renamed "Grain" section). Fixed-id, tool-global widgets forwarding over
//! the frozen `PanelEvent` channel to `PainterTool::set_brush_shape_*` / `set_brush_grain_depth`.
//! Split from `painter.rs` / `painter_texture.rs` to keep those under the workspace LOC cap.

use super::painter::fnv_node_id_runtime;
use super::{NodeId, hash_node_id};

/// **Grain Depth** slider (`0..1` track; `1` = full bite, default). `SetValue` → `set_brush_grain_depth`.
/// Lives in the Grain section (the renamed Texture section).
pub const PAINTER_BRUSH_GRAIN_DEPTH: NodeId = hash_node_id("painter_brush.grain_depth");

/// Shape **source** picker chip — `None` (the procedural Falloff) vs `Image` (an assigned silhouette).
/// Mirrors the Grain Kind picker: selecting `Image` opens a file pick (or use the Hierarchy "Use as
/// Brush Shape"); `None` clears the image. `SelectOption` → `set_brush_shape_kind`. Options via
/// [`painter_shape_kind_option_id`].
pub const PAINTER_SHAPE_KIND: NodeId = hash_node_id("painter_brush.shape_kind");

/// Derive the stable [`NodeId`] for Shape-source option `k` (the `TextureKind` wire discriminant —
/// only `None`/`Image` are offered) in the open Shape picker popover. Only the open popover's options
/// are hit-registered, so the `format!` is bounded.
#[must_use]
pub fn painter_shape_kind_option_id(k: u8) -> NodeId {
    fnv_node_id_runtime(&format!("painter_brush.shapekindopt.{k}"))
}

/// Collapsible **Shape** section header (Inspector pattern: ALL-CAPS label + collapse chevron +
/// assignable color dot). `mark_collapsible_section`-registered in `crate::populate`.
pub const PAINTER_SHAPE_SECTION: NodeId = hash_node_id("painter_brush.shape_section");
/// The Shape header's color dot — a picker swatch (`register_picker_swatch`).
pub const PAINTER_SHAPE_SECTION_COLOR: NodeId = hash_node_id("painter_brush.shape_section_color");
/// Shape section **reset** icon button — clears the Shape image (→ falloff) and resets the rotation
/// controls. `Click` → tool reset. Part of [`super::PAINTER_BRUSH_SECTION_RESETS`].
pub const PAINTER_SHAPE_RESET: NodeId = hash_node_id("painter_brush.shape_reset");

/// Shape **Angle** slider (`0..1` track → `0..=360°`). `SetValue` → `set_brush_shape_angle_norm`.
pub const PAINTER_SHAPE_ANGLE: NodeId = hash_node_id("painter_brush.shape_angle");
/// Shape **Rake** toggle (silhouette rotation follows the stroke). `Click` → `toggle_brush_shape_rake`.
pub const PAINTER_SHAPE_RAKE: NodeId = hash_node_id("painter_brush.shape_rake");
/// Shape **Random** toggle (random silhouette rotation per dab). `Click` → `toggle_brush_shape_random`.
pub const PAINTER_SHAPE_RANDOM: NodeId = hash_node_id("painter_brush.shape_random");
/// Shape **Offset X** slider (`0..1` track → `−1..1` tile). `SetValue` → `set_brush_shape_offset_norm(0, ..)`.
pub const PAINTER_SHAPE_OFFSET_X: NodeId = hash_node_id("painter_brush.shape_offset_x");
/// Shape **Offset Y** slider (axis 1). See [`PAINTER_SHAPE_OFFSET_X`].
pub const PAINTER_SHAPE_OFFSET_Y: NodeId = hash_node_id("painter_brush.shape_offset_y");
/// Shape **Size X** slider (`0..1` track → `0.1..10` scale). `SetValue` → `set_brush_shape_size_norm(0, ..)`.
pub const PAINTER_SHAPE_SIZE_X: NodeId = hash_node_id("painter_brush.shape_size_x");
/// Shape **Size Y** slider (axis 1). See [`PAINTER_SHAPE_SIZE_X`].
pub const PAINTER_SHAPE_SIZE_Y: NodeId = hash_node_id("painter_brush.shape_size_y");

/// The Shape + Grain-Depth **SetValue** sliders, as one array — a single membership check for the
/// panel's slider-drag forward (`event.rs`) and a single register loop (`populate.rs`).
pub const PAINTER_SHAPE_SLIDERS: [NodeId; 6] = [
    PAINTER_BRUSH_GRAIN_DEPTH,
    PAINTER_SHAPE_ANGLE,
    PAINTER_SHAPE_OFFSET_X,
    PAINTER_SHAPE_OFFSET_Y,
    PAINTER_SHAPE_SIZE_X,
    PAINTER_SHAPE_SIZE_Y,
];

// ── Shape **value ramp** (B&W tonal remap of the silhouette; "Shape Tone" section) ──────────────
/// Collapsible **Shape Tone** section header. `mark_collapsible_section`-registered in `crate::populate`.
pub const PAINTER_SHAPE_RAMP_SECTION: NodeId = hash_node_id("painter_brush.shape_ramp_section");
/// The Shape Tone header's colour dot (picker swatch).
pub const PAINTER_SHAPE_RAMP_SECTION_COLOR: NodeId =
    hash_node_id("painter_brush.shape_ramp_section_color");
/// Shape Tone section **reset** — ramp off + identity gradient. `Click` → `reset_shape_ramp`.
pub const PAINTER_SHAPE_RAMP_RESET: NodeId = hash_node_id("painter_brush.shape_ramp_reset");
/// "Use Shape Tone" enable toggle. `Click` → `toggle_shape_ramp_enabled`.
pub const PAINTER_SHAPE_RAMP_ENABLE: NodeId = hash_node_id("painter_brush.shape_ramp_enable");
/// Shape ramp **interpolation** dropdown. `SelectOption` → `set_shape_ramp_interp`.
pub const PAINTER_SHAPE_RAMP_INTERP: NodeId = hash_node_id("painter_brush.shape_ramp_interp");
/// The grayscale **bar** — the `CurvePoint` parent the draggable stop handles report against.
pub const PAINTER_SHAPE_RAMP_EDIT: NodeId = hash_node_id("painter_brush.shape_ramp_edit");
/// "+" add a stop. `Click` → `shape_ramp_add_stop`.
pub const PAINTER_SHAPE_RAMP_ADD: NodeId = hash_node_id("painter_brush.shape_ramp_add");
/// "−" remove the last stop. `Click` → `shape_ramp_remove_last_stop`.
pub const PAINTER_SHAPE_RAMP_REMOVE: NodeId = hash_node_id("painter_brush.shape_ramp_remove");
/// **Invert** button (flip stop positions L↔R). `Click` → `shape_ramp_invert`.
pub const PAINTER_SHAPE_RAMP_INVERT: NodeId = hash_node_id("painter_brush.shape_ramp_invert");
/// Selected-stop **index** chip (`NumberInput`).
pub const PAINTER_SHAPE_RAMP_STOP_INDEX: NodeId =
    hash_node_id("painter_brush.shape_ramp_stop_index");
/// Selected-stop **position** chip (`NumberInput`, `0..1`). `SelectOption` → `shape_ramp_move_stop`.
pub const PAINTER_SHAPE_RAMP_STOP_POS: NodeId = hash_node_id("painter_brush.shape_ramp_stop_pos");
/// Selected-stop **value** — the `SelectOption` target the **value bar** forwards to (`"id:value"`).
/// `SelectOption` → `shape_ramp_set_stop_value`. (No longer a `NumberInput`; see
/// [`painter_shape_ramp_value_handle_id`], Enio 2026-06-25.)
pub const PAINTER_SHAPE_RAMP_STOP_VALUE: NodeId =
    hash_node_id("painter_brush.shape_ramp_stop_value");
/// Stable [`NodeId`] for the grayscale **value bar**'s draggable square marker (one per selected stop
/// `i`). A `CurvePoint` whose parent is [`PAINTER_SHAPE_RAMP_EDIT`] with **channel `1`** (channel `0` =
/// the stop-position bar), so its drag rides the same `ValueChanged(EDIT)` route, distinguished by
/// channel (Enio 2026-06-25). A factory (paint-time-registered, like the stop handles), so the panel
/// wiring-parity gate treats it as the established `CurvePoint`-handle class.
#[must_use]
pub fn painter_shape_ramp_value_handle_id(i: u8) -> NodeId {
    fnv_node_id_runtime(&format!("painter_brush.shaperampvaluehandle.{i}"))
}

/// The Shape-ramp **Click** buttons (enable / add / remove / invert / reset) — forwarded as a
/// `PanelEvent::Click` by the panel's `event.rs` (a single membership check).
pub const PAINTER_SHAPE_RAMP_BUTTONS: [NodeId; 5] = [
    PAINTER_SHAPE_RAMP_ENABLE,
    PAINTER_SHAPE_RAMP_ADD,
    PAINTER_SHAPE_RAMP_REMOVE,
    PAINTER_SHAPE_RAMP_INVERT,
    PAINTER_SHAPE_RAMP_RESET,
];

/// The Shape-ramp **ValueChanged** ids (the position-bar + value-bar drags both report against
/// `PAINTER_SHAPE_RAMP_EDIT`, plus the editable index / position chips) — routed to `shape_ramp_picker`
/// by the panel's `event.rs` (a single membership check). The value bar forwards to
/// [`PAINTER_SHAPE_RAMP_STOP_VALUE`] (a `SelectOption`, not a `ValueChanged`).
pub const PAINTER_SHAPE_RAMP_VALUE_IDS: [NodeId; 3] = [
    PAINTER_SHAPE_RAMP_EDIT,
    PAINTER_SHAPE_RAMP_STOP_INDEX,
    PAINTER_SHAPE_RAMP_STOP_POS,
];

/// Stable [`NodeId`] for the draggable handle of Shape-ramp stop `i` (the open bar's stops are
/// hit-registered, so the `format!` is bounded).
#[must_use]
pub fn painter_shape_ramp_handle_id(i: u8) -> NodeId {
    fnv_node_id_runtime(&format!("painter_brush.shaperamphandle.{i}"))
}

/// Stable [`NodeId`] for Shape-ramp interpolation option `i` in the open dropdown popover.
#[must_use]
pub fn painter_shape_ramp_interp_option_id(i: u8) -> NodeId {
    fnv_node_id_runtime(&format!("painter_brush.shaperampinterpopt.{i}"))
}

/// Per-pattern parameter sliders for a **procedural** Shape (Contrast / Brightness + the kind's shape
/// knob), indexed by [`ph2d_painter_brush::TextureSettings::params`] slot — the Grain's
/// [`PAINTER_BRUSH_TEXTURE_PARAMS`](super::PAINTER_BRUSH_TEXTURE_PARAMS) twin, so the Shape gets every
/// characteristic param. `SetValue` → `set_brush_shape_param_norm`.
pub const PAINTER_SHAPE_PARAMS: [NodeId; 6] = [
    hash_node_id("painter_brush.shape_param_0"),
    hash_node_id("painter_brush.shape_param_1"),
    hash_node_id("painter_brush.shape_param_2"),
    hash_node_id("painter_brush.shape_param_3"),
    hash_node_id("painter_brush.shape_param_4"),
    hash_node_id("painter_brush.shape_param_5"),
];
