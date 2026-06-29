//! Drawing **Symmetry** section NodeIds (collapsible Brush-panel section that sits just above Tiling).
//! Fixed-id, tool-global widgets forwarding over the frozen `PanelEvent` channel to `PainterTool`'s
//! symmetry setters (`toggle_symmetry_*` / `set_symmetry_*` / `begin_symmetry_pick_*`, the single clamp
//! source). Split into its own file (like `painter_tiling.rs`) for the workspace LOC cap. Same
//! `PAINTER_BRUSH_*` prefix + public path (`ph2d_editor_core::ids::<NAME>`) via the chrome re-export.

use super::{NodeId, hash_node_id};

/// Collapsible "Symmetry" section header (Inspector pattern: ALL-CAPS label + reset + colour dot +
/// collapse chevron). Click toggles collapse; default collapsed (`crate::populate`).
pub const PAINTER_BRUSH_SYMMETRY_SECTION: NodeId = hash_node_id("painter_brush.symmetry_section");
/// The Symmetry header's assignable colour dot (a picker swatch, like the other section dots).
pub const PAINTER_BRUSH_SYMMETRY_SECTION_COLOR: NodeId =
    hash_node_id("painter_brush.symmetry_section_color");
/// Symmetry section **reset** icon button — restores the section to defaults. `Click` → `reset_symmetry`.
pub const PAINTER_BRUSH_SYMMETRY_RESET: NodeId = hash_node_id("painter_brush.symmetry_reset");

/// "Use Symmetry" master checkbox. `Click` → `toggle_symmetry_enabled`.
pub const PAINTER_BRUSH_SYMMETRY_USE: NodeId = hash_node_id("painter_brush.symmetry_use");
/// "Circular" checkbox (radial vs. mirror). `Click` → `toggle_symmetry_circular`.
pub const PAINTER_BRUSH_SYMMETRY_CIRCULAR: NodeId = hash_node_id("painter_brush.symmetry_circular");

/// Mirror-axis segmented option **X** (vertical mirror line). `Click` → `set_symmetry_axis(X)`.
pub const PAINTER_BRUSH_SYMMETRY_AXIS_X: NodeId = hash_node_id("painter_brush.symmetry_axis_x");
/// Mirror-axis segmented option **Y** (horizontal mirror line). `Click` → `set_symmetry_axis(Y)`.
pub const PAINTER_BRUSH_SYMMETRY_AXIS_Y: NodeId = hash_node_id("painter_brush.symmetry_axis_y");
/// Mirror-axis segmented option **Custom** (the artist-drawn line). `Click` → `set_symmetry_axis(Custom)`.
pub const PAINTER_BRUSH_SYMMETRY_AXIS_CUSTOM: NodeId =
    hash_node_id("painter_brush.symmetry_axis_custom");

/// The three axis segmented-control option ids `[X, Y, Custom]`, so the panel populate / paint /
/// dispatch can iterate without hardcoding each (the index is the `MirrorAxis::to_u8` discriminant).
pub const PAINTER_BRUSH_SYMMETRY_AXES: [NodeId; 3] = [
    PAINTER_BRUSH_SYMMETRY_AXIS_X,
    PAINTER_BRUSH_SYMMETRY_AXIS_Y,
    PAINTER_BRUSH_SYMMETRY_AXIS_CUSTOM,
];

/// Radial **segment-count** slider (`3..=12`). `SetValue` (the dispatched `0..1` track) →
/// `set_symmetry_segments`. Shown only when Circular is on.
pub const PAINTER_BRUSH_SYMMETRY_SEGMENTS: NodeId = hash_node_id("painter_brush.symmetry_segments");
/// Editable numeric chip paired with [`PAINTER_BRUSH_SYMMETRY_SEGMENTS`] (`link_slider_number_mapped_integer`).
pub const PAINTER_BRUSH_SYMMETRY_SEGMENTS_CHIP: NodeId =
    hash_node_id("painter_brush.symmetry_segments_chip");

/// "Draw Custom Line" mode-entry button (Custom axis). `Click` → `begin_symmetry_pick_line`.
pub const PAINTER_BRUSH_SYMMETRY_DRAW_LINE: NodeId =
    hash_node_id("painter_brush.symmetry_draw_line");
/// "Pick Center" mode-entry button (radial centre). `Click` → `begin_symmetry_pick_center`.
pub const PAINTER_BRUSH_SYMMETRY_PICK_CENTER: NodeId =
    hash_node_id("painter_brush.symmetry_pick_center");

/// Every Symmetry control that forwards a plain `PanelEvent::Click` (the toggles, the three axis
/// segments, the two pick buttons, and the section reset) — one membership check for the panel's
/// Click-forward and the tool's dispatch.
pub const PAINTER_BRUSH_SYMMETRY_CLICKABLE: [NodeId; 8] = [
    PAINTER_BRUSH_SYMMETRY_USE,
    PAINTER_BRUSH_SYMMETRY_CIRCULAR,
    PAINTER_BRUSH_SYMMETRY_AXIS_X,
    PAINTER_BRUSH_SYMMETRY_AXIS_Y,
    PAINTER_BRUSH_SYMMETRY_AXIS_CUSTOM,
    PAINTER_BRUSH_SYMMETRY_DRAW_LINE,
    PAINTER_BRUSH_SYMMETRY_PICK_CENTER,
    PAINTER_BRUSH_SYMMETRY_RESET,
];
