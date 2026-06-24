//! Seamless **Tiling** (wrap-around painting) section NodeIds. Fixed-id, tool-global widgets
//! forwarding over the frozen `PanelEvent` channel to `PainterTool::toggle_brush_tiling` (the single
//! clamp source). Split into its own file (like `painter_texture.rs`) for the workspace LOC cap;
//! Phase 2 (Repeat Image preview + Aspect Ratio X/Y) adds its ids here too.

use super::{NodeId, hash_node_id};

/// "Tiling X" toggle — wrap-around painting across the left/right sprite edges. `Click` →
/// `toggle_brush_tiling(0)`.
pub const PAINTER_BRUSH_TILING_X: NodeId = hash_node_id("painter_brush.tiling_x");
/// "Tiling Y" toggle — wrap-around painting across the top/bottom sprite edges. `Click` →
/// `toggle_brush_tiling(1)`.
pub const PAINTER_BRUSH_TILING_Y: NodeId = hash_node_id("painter_brush.tiling_y");

/// The Tiling toggle ids `[x, y]`, so the panel populate / dispatch can iterate without hardcoding
/// each (mirror of `PAINTER_BRUSH_TEXTURE_PARAMS`).
pub const PAINTER_BRUSH_TILING: [NodeId; 2] = [PAINTER_BRUSH_TILING_X, PAINTER_BRUSH_TILING_Y];

/// "Repeat Image" toggle — the on-canvas 3×3 tile preview (the sprite repeated in all directions).
/// `Click` → `toggle_repeat_image`.
pub const PAINTER_BRUSH_REPEAT_IMAGE: NodeId = hash_node_id("painter_brush.repeat_image");
/// Repeat-Image **Aspect Ratio X** slider (`0..1` → tile spacing ×sprite-width). `SetValue` →
/// `set_tile_aspect(0, _)`.
pub const PAINTER_BRUSH_TILE_ASPECT_X: NodeId = hash_node_id("painter_brush.tile_aspect_x");
/// Repeat-Image **Aspect Ratio Y** slider. `SetValue` → `set_tile_aspect(1, _)`.
pub const PAINTER_BRUSH_TILE_ASPECT_Y: NodeId = hash_node_id("painter_brush.tile_aspect_y");
/// The Repeat-Image aspect slider ids `[x, y]` (for the panel's `.contains` dispatch).
pub const PAINTER_BRUSH_TILE_ASPECT: [NodeId; 2] =
    [PAINTER_BRUSH_TILE_ASPECT_X, PAINTER_BRUSH_TILE_ASPECT_Y];
