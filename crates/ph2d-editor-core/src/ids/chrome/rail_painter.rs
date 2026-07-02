//! Left-rail **Painter mode** tool NodeIds.
//!
//! When the active image-edit tool is the Painter
//! ([`ImageEditState::active_tool_id`](crate::screens::hero::state::ImageEditState)
//! `== Some("painter")`), the rail swaps its object-mode transform block
//! (Translate / Rotate / Scale / Pivot) for these paint tools. They form an
//! exclusive radio group (`ButtonState::Pressed`), mirroring the transform
//! tools' dispatch in `chrome/rail_tools.rs` — clicking one activates it and
//! clears the rest. The **Shapes** button is part of the radio group but also
//! reveals a flyout of shape options to its right (press-and-hold).
//!
//! Selecting an image-edit tool BEHAVIOUR (wiring each tool to the Painter
//! engine) is a later step; this layer is the rail UI + selection state only.
use super::{NodeId, hash_node_id};

// ── Paint tools (rail order: Brush · Eyedropper · Eraser · Clone · Smear ·
//    Blur · Mask · Inpaint · Shapes) ─────────────────────────────────────────
/// Brush — the current paint mode (default-selected paint tool).
pub const PAINTER_RAIL_BRUSH: NodeId = hash_node_id("painter_rail.brush");
/// Fill (Bucket) — the flood-fill tool. Its rail chip is a COLOUR SWATCH (shows the current paint
/// colour) rather than an icon, so it doubles as the panel's colour selector. Sits below Brush, above
/// Eyedropper. Behaviour (fill + colour picker) wired in a follow-up.
pub const PAINTER_RAIL_FILL: NodeId = hash_node_id("painter_rail.fill");
/// Eyedropper — sample colour from the canvas (rich colour picker built in).
pub const PAINTER_RAIL_EYEDROPPER: NodeId = hash_node_id("painter_rail.eyedropper");
/// Eraser — erase to transparency (Erase-Alpha brush).
pub const PAINTER_RAIL_ERASER: NodeId = hash_node_id("painter_rail.eraser");
/// Clone — clone-stamp from a source point.
pub const PAINTER_RAIL_CLONE: NodeId = hash_node_id("painter_rail.clone");
/// Smear — smudge / blend / smear paint along the drag.
pub const PAINTER_RAIL_SMEAR: NodeId = hash_node_id("painter_rail.smear");
/// Blur — soften pixels under the brush.
pub const PAINTER_RAIL_BLUR: NodeId = hash_node_id("painter_rail.blur");
/// Mask — paint a selection / layer mask.
pub const PAINTER_RAIL_MASK: NodeId = hash_node_id("painter_rail.mask");
/// Inpaint — content-aware fill / heal.
pub const PAINTER_RAIL_INPAINT: NodeId = hash_node_id("painter_rail.inpaint");
/// Shapes — multi-shape button; press-and-hold reveals the shape flyout
/// ([`PAINTER_RAIL_SHAPE_IDS`]) to the right. A member of the tool radio group:
/// activating it puts the rail in "draw a shape" mode using the current shape.
pub const PAINTER_RAIL_SHAPES: NodeId = hash_node_id("painter_rail.shapes");

// ── Shape options (the Shapes flyout: Freehand · Line · Curve · Ellipse ·
//    Polygon) — a sub-radio; the Pressed one is the current shape ───────────
/// Free Hand stroke shape.
pub const PAINTER_RAIL_SHAPE_FREEHAND: NodeId = hash_node_id("painter_rail.shape_freehand");
/// Straight Line stroke shape.
pub const PAINTER_RAIL_SHAPE_LINE: NodeId = hash_node_id("painter_rail.shape_line");
/// Bézier Curve stroke shape.
pub const PAINTER_RAIL_SHAPE_CURVE: NodeId = hash_node_id("painter_rail.shape_curve");
/// Ellipse stroke shape.
pub const PAINTER_RAIL_SHAPE_ELLIPSE: NodeId = hash_node_id("painter_rail.shape_ellipse");
/// Polygon stroke shape.
pub const PAINTER_RAIL_SHAPE_POLYGON: NodeId = hash_node_id("painter_rail.shape_polygon");

/// Routing id (not a painted widget): the rail forwards the selected paint tool's mode to the active
/// Painter via `PanelEvent::SelectOption(PAINTER_PAINT_MODE, "brush" | "eraser" | "smear" | "blur")`,
/// drained by the shell into `PainterTool::handle_panel_event` → `set_paint_tool_mode`.
/// Dependency-legal (a frozen-channel message, no dep on the concrete painter crate).
pub const PAINTER_PAINT_MODE: NodeId = hash_node_id("painter_rail.paint_mode");

/// The Painter-mode tool radio group, in rail (paint) order. Exclusive
/// `ButtonState::Pressed` selection, like the transform tools. The Shapes
/// button is the last member (it also owns the shape flyout).
pub const PAINTER_RAIL_TOOL_IDS: [NodeId; 10] = [
    PAINTER_RAIL_BRUSH,
    PAINTER_RAIL_FILL,
    PAINTER_RAIL_EYEDROPPER,
    PAINTER_RAIL_ERASER,
    PAINTER_RAIL_CLONE,
    PAINTER_RAIL_SMEAR,
    PAINTER_RAIL_BLUR,
    PAINTER_RAIL_MASK,
    PAINTER_RAIL_INPAINT,
    PAINTER_RAIL_SHAPES,
];

/// The Shapes-flyout sub-radio, in flyout order. The Pressed one is the
/// current shape (used when the Shapes tool is active).
pub const PAINTER_RAIL_SHAPE_IDS: [NodeId; 5] = [
    PAINTER_RAIL_SHAPE_FREEHAND,
    PAINTER_RAIL_SHAPE_LINE,
    PAINTER_RAIL_SHAPE_CURVE,
    PAINTER_RAIL_SHAPE_ELLIPSE,
    PAINTER_RAIL_SHAPE_POLYGON,
];
