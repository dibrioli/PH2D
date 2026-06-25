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
