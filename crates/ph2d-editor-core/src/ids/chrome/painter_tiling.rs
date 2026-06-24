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

/// "Repeat Image" toggle — the on-canvas 3×3 tile preview (the sprite repeated in all directions);
/// with Tiling on, the neighbour tiles are paintable too. `Click` → `toggle_repeat_image`.
pub const PAINTER_BRUSH_REPEAT_IMAGE: NodeId = hash_node_id("painter_brush.repeat_image");
